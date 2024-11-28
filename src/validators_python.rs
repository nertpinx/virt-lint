/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::utils::*;
use crate::*;
use libxml::tree::Document;
use pyo3::exceptions::PyAttributeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

#[pyclass]
#[derive(Clone)]
struct ValidatorPython {
    vl: VirtLint,
    code: String,
    domxml: String,
    tags: Vec<String>,
}

impl ValidatorPython {
    pub fn from_path<T: AsRef<Path>, S: AsRef<Path>>(
        path: T,
        prefix: S,
        vl: &mut VirtLint,
        domxml: String,
    ) -> VirtLintResult<Self> {
        let tags = get_tags_for_path(prefix, &path);
        let vl = vl.clone();
        let code = std::fs::read_to_string(path)?;

        Ok(Self {
            vl,
            code,
            domxml,
            tags,
        })
    }

    pub fn validate(&mut self) -> VirtLintResult<()> {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| -> PyResult<()> {
            let globals = PyDict::new_bound(py);
            globals.set_item("vl", Py::new(py, self.clone())?)?;

            let output = py.run_bound(&self.code, Some(&globals), None);

            if let Err(ref err) = output {
                if let Some(tb) = err.traceback_bound(py) {
                    eprint!("{}", tb.format()?);
                }
            }

            globals.del_item("vl")?;

            output
        })?;

        Ok(())
    }
}

#[pymethods]
impl ValidatorPython {
    fn dom_xpath(&mut self, xpath: String) -> Result<Option<Vec<String>>, VirtLintError> {
        let parser = Parser::default();
        let doc = parser.parse_string(&self.domxml)?;
        Ok(xpath_eval_nodeset_or_none(&doc, &xpath))
    }

    fn add_warning(&mut self, domain: i32, level: i32, msg: String) -> Result<(), VirtLintError> {
        let domain = WarningDomain::try_from(domain)?;
        let level = WarningLevel::try_from(level)?;

        self.vl
            .add_warning(self.tags.clone(), domain, level, msg);

        Ok(())
    }

    fn caps_xpath(&mut self, xpath: String) -> Result<Option<Vec<String>>, VirtLintError> {
        let caps = match self.vl.capabilities_get()? {
            Some(caps) => caps,
            None => {
                return Ok(None);
            }
        };

        let parser = Parser::default();
        let caps_doc = parser.parse_string(caps)?;

        Ok(xpath_eval_nodeset_or_none(&caps_doc, &xpath))
    }

    fn domcaps_xpath(&mut self, xpath: String) -> Result<Option<Vec<String>>, VirtLintError> {
        let parser = Parser::default();
        let dom_doc = parser.parse_string(&self.domxml)?;

        let domcaps = match self.vl.domain_capabilities_get(Some(&dom_doc))? {
            Some(domcaps) => domcaps,
            None => {
                return Ok(None);
            }
        };

        let domcaps_doc = parser.parse_string(domcaps)?;

        Ok(xpath_eval_nodeset_or_none(&domcaps_doc, &xpath))
    }

    fn get_libvirt_conn(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        let conn = match self
            .vl
            .get_virt_conn()?
        {
            None => return Ok(None),
            Some(c) => c,
        };

        let ptr = conn.as_ptr();
        let py_ptr = ptr.cast::<virt_sys::virConnectPtr>() as *mut std::ffi::c_void;
        let name = pyo3::ffi::c_str!("virConnectPtr");
        let capsule = unsafe { pyo3::ffi::PyCapsule_New(py_ptr, name.as_ptr(), None) };
        let capsule_bound = unsafe { Bound::from_owned_ptr(py, capsule) };
        let libvirt_python = py.import_bound("libvirt")?;
        let class = libvirt_python.getattr("virConnect")?;

        class.call1((capsule_bound,))?.extract()
    }

    // Ehm, I was too lazy to wrap the i32
    fn __getattr__(&self, name: &str) -> PyResult<i32> {
        match name {
            "WarningDomain_Domain" => Ok(WarningDomain::Domain as i32),
            "WarningDomain_Node" => Ok(WarningDomain::Node as i32),
            "WarningLevel_Error" => Ok(WarningLevel::Error as i32),
            "WarningLevel_Warning" => Ok(WarningLevel::Warning as i32),
            "WarningLevel_Notice" => Ok(WarningLevel::Notice as i32),
            _ => Err(PyAttributeError::new_err(format!(
                "'vl' object has no attribute '{name}'"
            ))),
        }
    }
}

fn get_tags_for_path<P: AsRef<Path>>(prefix: P, path: impl AsRef<Path>) -> Vec<String> {
    let mut ret = Vec::new();

    let p = match path.as_ref().strip_prefix(prefix.as_ref()) {
        Ok(x) => x,
        Err(_) => return vec![],
    };

    for anc in p.ancestors() {
        match PathBuf::from(anc)
            .with_extension("")
            .into_os_string()
            .into_string()
        {
            Ok(x) => {
                if !x.is_empty() {
                    ret.push(x);
                }
            }
            Err(_) => continue,
        }
    }

    ret
}

fn get_paths_for_tag(
    prefix: impl AsRef<Path>,
    tag: &String,
    filename_prefix: &OsString,
    ext: &OsString,
) -> VirtLintResult<Vec<PathBuf>> {
    let path = prefix.as_ref().join(tag);

    if !path.is_dir() {
        let path = path.with_extension(ext);
        if path.exists() {
            return Ok(vec![path]);
        }
    }

    recurse_files(path, Some(filename_prefix), Some(ext))
}

fn get_validators(
    prefix: &PathBuf,
    tags: &[String],
    filename_prefix: &OsString,
    ext: &OsString,
) -> Vec<PathBuf> {
    let mut ret: HashSet<PathBuf> = HashSet::new();

    if tags.is_empty() {
        return recurse_files(prefix, Some(filename_prefix), Some(ext)).unwrap_or_default();
    } else {
        for tag in tags.iter() {
            let tag_paths =
                get_paths_for_tag(prefix, tag, filename_prefix, ext).unwrap_or_default();

            for tag_path in tag_paths {
                ret.insert(tag_path);
            }
        }
    }

    let mut ret = ret.into_iter().collect::<Vec<PathBuf>>();
    ret.sort();
    ret
}

pub struct ValidatorsPython {
    prefix: Vec<PathBuf>,
    filename_prefix: OsString,
    ext: OsString,
}

impl ValidatorsPython {
    pub fn new(prefix: &[PathBuf], filename_prefix: &'static str, ext: &'static str) -> Self {
        let prefix = prefix.iter().filter(|p| p.exists()).cloned().collect();

        Self {
            prefix,
            filename_prefix: OsString::from(filename_prefix),
            ext: OsString::from(ext),
        }
    }

    pub fn list_tags(&self) -> VirtLintResult<HashSet<String>> {
        let mut ret: HashSet<String> = HashSet::new();

        for p in &self.prefix {
            let rc = recurse_files(p, Some(&self.filename_prefix), Some(&self.ext))?;
            for path in rc {
                let tags = get_tags_for_path(p, &path);

                for tag in tags {
                    ret.insert(tag);
                }
            }
        }

        Ok(ret)
    }

    pub fn validate(
        &self,
        tags: &[String],
        vl: &mut VirtLint,
        domxml: &str,
        _domxml_doc: &Document,
    ) -> VirtLintResult<()> {
        let mut validators = HashMap::new();

        for p in &self.prefix {
            let paths = get_validators(p, tags, &self.filename_prefix, &self.ext);

            for path in paths {
                let subpath = path.strip_prefix(p).expect("Internal impossible error");

                validators.entry(subpath.to_owned()).or_insert((path, p));
            }
        }

        for (path, p) in validators.values() {
            ValidatorPython::from_path(path, p, vl, domxml.to_string())?.validate()?;
        }

        Ok(())
    }
}
