//! Namespace descriptor system for organizing API methods and types

use super::type_system::TypeDescriptor;

/// Describes a namespace in the API system
#[derive(Debug, Clone)]
pub struct NamespaceDescriptor {
    pub name: String,
    pub parent: Option<String>,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub classes: Vec<ClassDescriptor>,
    pub child_namespaces: Vec<String>,
    pub documentation: String,
}

impl NamespaceDescriptor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parent: None,
            methods: Vec::new(),
            properties: Vec::new(),
            classes: Vec::new(),
            child_namespaces: Vec::new(),
            documentation: String::new(),
        }
    }

    pub fn with_parent(mut self, parent: String) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_methods(mut self, methods: Vec<String>) -> Self {
        self.methods = methods;
        self
    }

    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = documentation;
        self
    }

    pub fn with_classes(mut self, classes: Vec<ClassDescriptor>) -> Self {
        self.classes = classes;
        self
    }

    pub fn add_method(&mut self, method_name: String) {
        self.methods.push(method_name);
    }

    pub fn add_class(&mut self, class: ClassDescriptor) {
        self.classes.push(class);
    }
}

/// Describes a class within a namespace
#[derive(Debug, Clone)]
pub struct ClassDescriptor {
    pub name: String,
    pub methods: Vec<String>,
    pub properties: Vec<PropertyDescriptor>,
    pub constructor: Option<String>,
    pub documentation: String,
}

impl ClassDescriptor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: Vec::new(),
            properties: Vec::new(),
            constructor: None,
            documentation: String::new(),
        }
    }

    pub fn with_methods(mut self, methods: Vec<String>) -> Self {
        self.methods = methods;
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDescriptor>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = documentation;
        self
    }

    pub fn add_method(&mut self, method_name: String) {
        self.methods.push(method_name);
    }

    pub fn add_property(&mut self, property: PropertyDescriptor) {
        self.properties.push(property);
    }
}

/// Describes a property within a class
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: String,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub readonly: bool,
    pub property_type: TypeDescriptor,
    pub documentation: String,
}

impl PropertyDescriptor {
    pub fn new(name: String, property_type: TypeDescriptor) -> Self {
        Self {
            name,
            getter: None,
            setter: None,
            readonly: false,
            property_type,
            documentation: String::new(),
        }
    }

    pub fn with_getter(mut self, getter: String) -> Self {
        self.getter = Some(getter);
        self
    }

    pub fn with_setter(mut self, setter: String) -> Self {
        self.setter = Some(setter);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = documentation;
        self
    }
}