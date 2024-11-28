/* SPDX-License-Identifier: LGPL-3.0-or-later */

// The rustc is complaining about dead code because only used when
// ignored tests are executed.

#[cfg(test)]
use crate::*;
use std::sync::Once;
use virt::connect::Connect;
use virt::domain::Domain;

static TEST_INIT: Once = Once::new();

fn test_init() {
    TEST_INIT.call_once(|| {
        // Set
        std::env::set_var(
            "VIRT_LINT_VALIDATORS_PATH",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../validators"),
        )
    });
}

fn conn() -> Connect {
    Connect::open(Some("test:///default")).unwrap()
}

fn close(mut conn: Connect) {
    assert_eq!(Ok(0), conn.close(), "close(), expected 0")
}

#[test]
fn test_empty() {
    test_init();

    let c = conn();
    {
        let vl = VirtLint::new(Some(&c));
        assert!(vl.warnings().is_empty());
    }
    close(c);
}

#[test]
fn test_list_tags() {
    test_init();

    let tags = VirtLint::list_validator_tags().unwrap();
    assert_eq!(
        tags,
        [
            "TAG_1",
            "TAG_2",
            "TAG_3",
            "TAG_4",
            "common",
            "common/check_node_kvm",
            "common/check_numa",
            "common/check_numa_free",
            "common/check_pcie_root_ports",
            "common_p",
            "common_p/check_node_kvm",
            "common_p/check_numa",
            "common_p/check_numa_free",
            "common_p/check_pcie_root_ports",
        ]
    );
}

#[test]
fn test_simple() {
    test_init();

    let c = conn();
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        let domxml = dom.get_xml_desc(0).unwrap_or_default();

        let mut vl = VirtLint::new(Some(&c));

        vl.validate(&domxml, &[], false).unwrap();

        let mut warnings = vl.warnings();
        warnings.sort();

        assert_eq!(
            warnings,
            vec![
                VirtLintWarning::new(
                    vec![String::from("TAG_1"), String::from("TAG_2")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("TAG_2")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("common"), String::from("common/check_numa")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![
                        String::from("common"),
                        String::from("common/check_numa_free")
                    ],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("common_p"), String::from("common_p/check_numa")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![
                        String::from("common_p"),
                        String::from("common_p/check_numa_free")
                    ],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
            ]
        );
    }

    close(c);
}

#[test]#[ignore = "pollutes environment for other tests"]
fn test_override() {
    let old_path = std::env::var_os("VIRT_LINT_VALIDATORS_PATH");
    std::env::set_var(
        "VIRT_LINT_VALIDATORS_PATH",
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../validators_overrides"),
            ":",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../validators"),
        )
    );

    let c = conn();
    {
        let dom = Domain::lookup_by_name(&c, "test").unwrap();

        let domxml = dom.get_xml_desc(0).unwrap_or_default();

        let mut vl = VirtLint::new(Some(&c));

        let res = vl.validate(&domxml, &["common_p".to_string()], false);

        if let Some(old_path) = old_path {
            std::env::set_var("VIRT_LINT_VALIDATORS_PATH", old_path);
        }

        res.unwrap();

        assert_eq!(
            vl.warnings(),
            vec![
                VirtLintWarning::new(
                    vec![
                        String::from("common_p"),
                        String::from("common_p/check_numa_free")
                    ],
                    WarningDomain::Node,
                    WarningLevel::Notice,
                    String::from("Different message")
                ),
            ]
        );
    }

    close(c);
}

#[test]
fn test_offline_simple() {
    test_init();

    // The connection here is used only to get domain XML and capabilities. Validation is done
    // completely offline.
    let c = conn();
    let domxml;
    let capsxml;
    let domcapsxml;
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        domxml = dom.get_xml_desc(0).unwrap_or_default();
        capsxml = c.get_capabilities().unwrap_or_default();
        domcapsxml = c
            .get_domain_capabilities(None, None, None, None, 0)
            .unwrap_or_default();
    }

    close(c);

    let mut vl = VirtLint::new(None);

    vl.capabilities_set(Some(capsxml)).unwrap();
    vl.domain_capabilities_add(domcapsxml).unwrap();
    vl.validate(&domxml, &[], false).unwrap();

    let mut warnings = vl.warnings();

    warnings.sort();

    assert_eq!(
        warnings,
        vec![
            VirtLintWarning::new(
                vec![String::from("TAG_1"), String::from("TAG_2")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common"), String::from("common/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common_p"), String::from("common_p/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
        ]
    );
}

#[test]
fn test_offline_with_error() {
    test_init();

    // The connection here is used only to get domain XML and capabilities. Validation is done
    // completely offline.
    let c = conn();
    let domxml;
    let capsxml;
    let domcapsxml;
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        domxml = dom.get_xml_desc(0).unwrap_or_default();
        capsxml = c.get_capabilities().unwrap_or_default();
        domcapsxml = c
            .get_domain_capabilities(None, None, None, None, 0)
            .unwrap_or_default();
    }

    close(c);

    let mut vl = VirtLint::new(None);

    vl.capabilities_set(Some(capsxml)).unwrap();
    vl.domain_capabilities_add(domcapsxml).unwrap();

    // This fails, because there is a validator that requires connection
    assert!(vl.validate(&domxml, &[], true).is_err());

    // This succeeds, because we deliberately run offline only validators
    vl.validate(
        &domxml,
        &vec![
            String::from("TAG_1"),
            String::from("TAG_3"),
            String::from("TAG_4"),
            String::from("common/check_node_kvm"),
            String::from("common/check_numa"),
            String::from("common/check_pcie_root_ports"),
            String::from("common_p/check_node_kvm"),
            String::from("common_p/check_numa"),
            String::from("common_p/check_pcie_root_ports"),
        ],
        true
    ).unwrap();

    let mut warnings = vl.warnings();

    warnings.sort();

    assert_eq!(
        warnings,
        vec![
            VirtLintWarning::new(
                vec![String::from("TAG_1"), String::from("TAG_2")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common"), String::from("common/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common_p"), String::from("common_p/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
        ]
    );
}
