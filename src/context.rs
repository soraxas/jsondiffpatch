use crate::types::Options;
use serde_json::Value;

pub trait Context {
    fn pipe(&self) -> &str;
    fn set_result(&mut self, result: Value);
    fn exit(&mut self);
    fn push(&mut self, child: Box<dyn Context>, name: Option<String>);
    fn get_options(&self) -> &Options;
    fn set_options(&mut self, options: Options);
    fn get_result(&self) -> Option<&Value>;
    fn has_result(&self) -> bool;
    fn is_exiting(&self) -> bool;
    fn get_parent(&self) -> Option<&Box<dyn Context>>;
    fn get_child_name(&self) -> Option<&String>;
    fn get_root(&self) -> Option<&Box<dyn Context>>;
    fn get_children(&self) -> Option<&Vec<Box<dyn Context>>>;
    fn get_next_after_children(&self) -> Option<&Box<dyn Context>>;
    fn get_next(&self) -> Option<&Box<dyn Context>>;
    fn set_next(&mut self, next: Option<Box<dyn Context>>);
    fn set_next_after_children(&mut self, next: Option<Box<dyn Context>>);
}

pub struct BaseContext {
    pub pipe_name: String,
    pub result: Option<Value>,
    pub has_result: bool,
    pub exiting: bool,
    pub parent: Option<Box<dyn Context>>,
    pub child_name: Option<String>,
    pub root: Option<Box<dyn Context>>,
    pub options: Options,
    pub children: Option<Vec<Box<dyn Context>>>,
    pub next_after_children: Option<Box<dyn Context>>,
    pub next: Option<Box<dyn Context>>,
}

impl BaseContext {
    pub fn new(pipe_name: String) -> Self {
        Self {
            pipe_name,
            result: None,
            has_result: false,
            exiting: false,
            parent: None,
            child_name: None,
            root: None,
            options: Options::default(),
            children: None,
            next_after_children: None,
            next: None,
        }
    }
}

impl Context for BaseContext {
    fn pipe(&self) -> &str {
        &self.pipe_name
    }

    fn set_result(&mut self, result: Value) {
        self.result = Some(result);
        self.has_result = true;
    }

    fn exit(&mut self) {
        self.exiting = true;
    }

    fn push(&mut self, child: Box<dyn Context>, name: Option<String>) {
        // Set child's parent
        // Note: This is a simplified implementation
        // In a full implementation, we'd need to handle the parent reference properly

        if let Some(name) = name {
            // Set child name if provided
        }

        // Set root reference
        let root = self.root.as_ref().map(|r| r.clone()).unwrap_or_else(|| {
            // Create a reference to self as root
            // This is simplified - in practice we'd need proper reference handling
            Box::new(BaseContext::new("root".to_string())) as Box<dyn Context>
        });

        // Set options
        // Note: This is simplified - we'd need to properly share options

        // Add to children list
        if self.children.is_none() {
            self.children = Some(vec![child]);
            self.next_after_children = self.next.take();
            // Set next to child (simplified)
        } else {
            if let Some(children) = &mut self.children {
                // Set the last child's next to this new child
                if let Some(last_child) = children.last_mut() {
                    last_child.set_next(Some(child));
                }
                children.push(child);
            }
        }
    }

    fn get_options(&self) -> &Options {
        &self.options
    }

    fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    fn get_result(&self) -> Option<&Value> {
        self.result.as_ref()
    }

    fn has_result(&self) -> bool {
        self.has_result
    }

    fn is_exiting(&self) -> bool {
        self.exiting
    }

    fn get_parent(&self) -> Option<&Box<dyn Context>> {
        self.parent.as_ref()
    }

    fn get_child_name(&self) -> Option<&String> {
        self.child_name.as_ref()
    }

    fn get_root(&self) -> Option<&Box<dyn Context>> {
        self.root.as_ref()
    }

    fn get_children(&self) -> Option<&Vec<Box<dyn Context>>> {
        self.children.as_ref()
    }

    fn get_next_after_children(&self) -> Option<&Box<dyn Context>> {
        self.next_after_children.as_ref()
    }

    fn get_next(&self) -> Option<&Box<dyn Context>> {
        self.next.as_ref()
    }

    fn set_next(&mut self, next: Option<Box<dyn Context>>) {
        self.next = next;
    }

    fn set_next_after_children(&mut self, next: Option<Box<dyn Context>>) {
        self.next_after_children = next;
    }
}