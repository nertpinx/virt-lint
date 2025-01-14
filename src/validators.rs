/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::utils::*;
use crate::validators_lua::*;
use crate::validators_python::*;
use crate::*;
use libxml::parser::Parser;
use libxml::tree::Document;
use libxml::xpath::Context;
use std::collections::HashSet;
use std::path::PathBuf;

type ValidatorCB = dyn Fn(&mut VirtLint, &str, &Document, &Validator) -> VirtLintResult<()>;

struct Validator {
    cb: &'static ValidatorCB,
    tags: HashSet<&'static str>,
}

pub struct Validators {
    validators: Vec<Validator>,
    lua: ValidatorsLua,
    python: ValidatorsPython,
}

impl Validators {
    fn new_paths(paths: Vec<PathBuf>) -> Self {
        let validators = vec![
            Validator {
                cb: &check_numa,
                tags: HashSet::from(["TAG_1", "TAG_2"]),
            },
            Validator {
                cb: &check_numa_free,
                tags: HashSet::from(["TAG_2"]),
            },
            Validator {
                cb: &check_node_kvm,
                tags: HashSet::from(["TAG_1", "TAG_3"]),
            },
            Validator {
                cb: &check_pcie_root_ports,
                tags: HashSet::from(["TAG_4"]),
            },
        ];

        let lua = ValidatorsLua::new(&paths, "check_", "lua");
        let python = ValidatorsPython::new(&paths, "check_", "py");

        Self {
            validators,
            lua,
            python,
        }
    }

    pub fn new() -> Self {
        let mut paths = Vec::new();

        if let Some(user_paths) = std::env::var_os("VIRT_LINT_VALIDATORS_PATH") {
            paths.extend(std::env::split_paths(&user_paths))
        } else {
            paths.extend([
                PathBuf::from("./validators"),
                PathBuf::from("/etc/virt-lint/validators"),
                PathBuf::from("/usr/share/virt-lint/validators"),
            ]);
        }

        Self::new_paths(paths)
    }

    pub fn list_tags(&mut self) -> VirtLintResult<HashSet<String>> {
        let mut tags: HashSet<String> = HashSet::new();

        for v in &self.validators {
            v.tags.iter().for_each(|t| {
                tags.insert(t.to_string());
            });
        }

        tags.extend(self.lua.list_tags()?);
        tags.extend(self.python.list_tags()?);

        Ok(tags)
    }

    fn validate_tags(&mut self, tags: &[String]) -> VirtLintResult<()> {
        let known_tags: HashSet<String> = self.list_tags()?;

        for tag in tags.iter() {
            if !known_tags.contains(tag) {
                return Err(VirtLintError::UnknownValidatorTag(tag.to_string()));
            }
        }

        Ok(())
    }

    fn get_validators(&self, tags: &[String]) -> Vec<&Validator> {
        if tags.is_empty() {
            self.validators.iter().collect()
        } else {
            /*
             * Cannot do `.contains() on a slice of &String with &'static str
             * hence the .iter().any(|it| it == t)
             * Cannot do `.contains() on a hashset of &'static str with String
             * hence the .iter().any(|t| tags.iter().any(|it| it == t)))
             *
             * See https://doc.rust-lang.org/std/primitive.slice.html#method.contains
             */
            self.validators
                .iter()
                .filter(|v| v.tags.iter().any(|t| tags.iter().any(|it| it == t)))
                .collect()
        }
    }

    pub fn validate(
        &mut self,
        tags: &[String],
        vl: &mut VirtLint,
        domxml: &str,
    ) -> VirtLintResult<()> {
        let parser = Parser::default();
        let domxml_doc = parser.parse_string(domxml)?;

        self.validate_tags(tags)?;

        let validators = self.get_validators(tags);

        self.lua.validate(tags, vl, domxml, &domxml_doc)?;

        self.python.validate(tags, vl, domxml, &domxml_doc)?;

        for validator in validators.iter() {
            (validator.cb)(vl, domxml, &domxml_doc, validator)?;
        }

        Ok(())
    }
}

fn check_numa(
    vl: &mut VirtLint,
    _domxml: &str,
    domxml_doc: &Document,
    va: &Validator,
) -> VirtLintResult<()> {
    let mut numa_mems: Vec<u64> = Vec::new();
    let mut dom_mem: u64 = 0;
    let mut would_fit: bool = false;
    let parser = Parser::default();

    let caps = match vl.capabilities_get()? {
        Some(caps) => parser.parse_string(caps)?,
        None => {
            return Ok(());
        }
    };

    let ctxt = Context::new(&caps).unwrap();

    let nodes = ctxt
        .evaluate("//capabilities/host/topology/cells/cell/memory/text()")
        .unwrap();

    for node in &nodes.get_nodes_as_vec() {
        numa_mems.push(node.get_content().parse().unwrap())
    }

    if let Some(mem) = xpath_eval_or_none(domxml_doc, "//domain/memory") {
        dom_mem = parse_int(&mem)?
    }

    for node in numa_mems.iter() {
        if node > &dom_mem {
            would_fit = true;
            break;
        }
    }

    if !would_fit {
        vl.add_warning(
            va.tags
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Domain would not fit into any host NUMA node"),
        );
    }

    Ok(())
}

fn check_numa_free(
    vl: &mut VirtLint,
    _domxml: &str,
    domxml_doc: &Document,
    va: &Validator,
) -> VirtLintResult<()> {
    let mut numa_ids: Vec<i32> = Vec::new();
    let mut numa_mems_free: Vec<u64> = Vec::new();
    let mut dom_mem: u64 = 0;
    let mut would_fit: bool = false;
    let parser = Parser::default();

    let conn = match vl.get_conn()? {
        Some(c) => c,
        None => return Ok(()),
    };

    let caps = match vl.capabilities_get()? {
        Some(caps) => parser.parse_string(caps)?,
        None => {
            return Ok(());
        }
    };

    let ctxt = Context::new(&caps).unwrap();

    let nodes = ctxt
        .evaluate("//capabilities/host/topology/cells/cell/@id")
        .unwrap();

    for node in nodes.get_nodes_as_vec() {
        numa_ids.push(node.get_content().parse().unwrap())
    }

    for node in numa_ids.iter() {
        conn.conn
            .get_cells_free_memory(*node, 1)
            .unwrap()
            .into_iter()
            .for_each(|x| numa_mems_free.push(x));
    }

    if let Some(mem) = xpath_eval_or_none(domxml_doc, "//domain/memory") {
        dom_mem = parse_int(&mem)?
    }

    numa_mems_free.into_iter().for_each(|x| {
        if x > dom_mem {
            would_fit = true;
        }
    });

    if !would_fit {
        vl.add_warning(
            va.tags
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Not enough free memory on any NUMA node"),
        );
    }

    Ok(())
}

fn check_node_kvm(
    vl: &mut VirtLint,
    _domxml: &str,
    domxml_doc: &Document,
    va: &Validator,
) -> VirtLintResult<()> {
    let mut emit_warning = false;

    if vl.domain_capabilities_get(Some(domxml_doc)).is_err() {
        emit_warning = true;
        /* Plain fact we failed to look up domain capabilities for given XML warrants a warning.
         * But let's try harder. */
    }

    if ! emit_warning {
        return Ok(());
    }

    if let Some(caps) = vl.capabilities_get()? {
        let parser = Parser::default();
        let caps = parser.parse_string(caps)?;
        let mut xpath: String = String::new();

        let emulator = xpath_eval_or_none(&caps, "//domain/devices/emulator");
        let arch = xpath_eval_or_none(&caps, "//domain/os/type/@arch");
        let machine = xpath_eval_or_none(&caps, "//domain/os/type/@machine");
        let virttype = xpath_eval_or_none(&caps, "//domain/@type");

        if let Some(s) = arch {
            xpath += &format!("@name='{s}'")
        }

        if let Some(s) = emulator {
            xpath += &format!(
                "{}emulator/text()='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        if let Some(s) = machine {
            xpath += &format!(
                "{}machine/text()='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        if let Some(s) = virttype {
            xpath += &format!(
                "{}domain/@type='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        let mut top_xpath = String::from("//capabilities/guest/arch");
        if !xpath.is_empty() {
            top_xpath += &format!("[{xpath}]")
        };

        emit_warning = xpath_eval_or_none(&caps, &top_xpath).is_none();
    }

    if emit_warning {
        vl.add_warning(
            va.tags
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            WarningDomain::Node,
            WarningLevel::Warning,
            String::from("No suitable emulator found"),
        );
    }

    Ok(())
}

fn check_pcie_root_ports(
    vl: &mut VirtLint,
    _domxml: &str,
    domxml_doc: &Document,
    va: &Validator,
) -> VirtLintResult<()> {
    let mut pcie_chassis: Vec<u64> = Vec::new();

    let virttype = match xpath_eval_or_none(domxml_doc, "//domain/@type") {
        Some(x) => x,
        None => {
            return Ok(());
        }
    };

    if virttype != "kvm" && virttype != "qemu" {
        return Ok(());
    }

    let machine = match xpath_eval_or_none(domxml_doc, "//domain/os/type/@machine") {
        Some(x) => x,
        None => {
            return Ok(());
        }
    };

    if !machine.contains("q35") {
        return Ok(());
    }

    let ctxt = Context::new(domxml_doc).unwrap();

    let nodes = ctxt
        .evaluate("//domain/devices/controller[@type='pci']/target/@chassis")
        .unwrap();

    for node in &nodes.get_nodes_as_vec() {
        pcie_chassis.push(node.get_content().parse().unwrap());
    }

    // Firstly, remove obviously taken root ports
    if !pcie_chassis.is_empty() {
        let nodes = ctxt
            .evaluate("//domain/devices//address[@type='pci']/@bus")
            .unwrap();

        for node in &nodes.get_nodes_as_vec() {
            let bus: u64 = parse_int(&node.get_content())?;
            let mut i = 0;

            while i < pcie_chassis.len() {
                if pcie_chassis[i] != bus {
                    i += 1;
                    continue;
                }

                pcie_chassis.remove(i);
            }
        }
    }

    // Then, remove those, which would be taken by PCI address auto assignment
    // TODO

    if pcie_chassis.is_empty() {
        vl.add_warning(
            va.tags
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            WarningDomain::Domain,
            WarningLevel::Notice,
            String::from("No free PCIe root ports found, hotplug might be not possible"),
        );
    }

    Ok(())
}
