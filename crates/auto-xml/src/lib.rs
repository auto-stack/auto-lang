use auto_val::{AutoStr, Node};

pub trait ToXML {
    fn to_xml(&self) -> AutoStr;
}

impl ToXML for Node {
    fn to_xml(&self) -> AutoStr {
        // node name -> xml tag name
        let name = self.name.clone();

        if name == "text" {
            return self.text.clone();
        }

        let mut xml = String::new();

        // start tag
        xml.push_str(format!("<{}", name).as_str());

        // fill props
        for (k, v) in self.props.iter() {
            xml.push_str(format!(" {}=\"{}\"", k, v.repr()).as_str());
        }

        if self.nodes.is_empty() {
            xml.push_str("/>")
        } else {
            xml.push_str(">");
            // fill kids
            for k in self.nodes.iter() {
                xml.push_str(k.to_xml().as_str());
            }

            // end tag
            xml.push_str(format!("</{}>", name).as_str());
        }

        xml.into()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use auto_lang::config::AutoConfig;
    use auto_val::Value;

    #[test]
    fn test_node_to_xml() {
        let mut node = Node::new("test");
        node.set_prop("name", "ming");
        node.set_prop("age", Value::Int(12));

        let mut kid = Node::new("score");
        kid.set_prop("name", "Math");
        kid.set_prop("score", Value::Int(145));
        node.add_kid(kid);
        let xml = node.to_xml();
        let expected = r#"
            <test age="12" name="ming">
                <score name="Math" score="145"></score>
            </test>
        "#;
        let expected = auto_lang::util::compact_xml(expected).unwrap();
        assert_eq!(xml, expected);
    }

    #[test]
    fn test_config_to_xml() {
        let xml = r#"
            <root>
                <group>
                    <name>App</name>
                    <group>
                        <name>os</name>
                        <group>
                            <name>modules</name>
                            <file>module1.rs</file>
                            <file>module2.rs</file>
                        </group>
                        <group>
                            <name>config</name>
                            <file>config1.rs</file>
                            <file>config2.rs</file>
                        </group>
                    </group>
                </group>
            </root>
        "#;
        let xml = auto_lang::util::compact_xml(xml).unwrap();

        let config = r#"
            group {
                name {"App"}
                group {
                    name {"os"}
                    group {
                        name {"modules"}
                        file {"module1.rs"}
                        file {"module2.rs"}
                    }
                    group {
                        name {"config"}
                        file {"config1.rs"}
                        file {"config2.rs"}
                    }
                }
            }
        "#;

        let cfg = AutoConfig::new(config).unwrap();
        let node = cfg.root;
        let node_xml = node.to_xml();
        assert_eq!(node_xml, xml);
        // let config = AutoConfig::from_code(xml, &Obj::default()).unwrap();
        // assert_eq!(config.name(), "hello");
        // assert_eq!(config.list_target_names(), vec!["lib(\"alib\")"]);
    }
}
