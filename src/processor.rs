use crate::context::Context;
use crate::types::Options;
use serde_json::Value;
use std::collections::HashMap;

pub trait Filter: Send + Sync {
    fn filter_name(&self) -> &str;
    fn process(&self, context: &mut Box<dyn Context>);
}

pub struct Pipe {
    pub name: String,
    pub processor: Option<Box<Processor>>,
    pub filters: Vec<Box<dyn Filter>>,
    pub should_have_result: bool,
}

impl Pipe {
    pub fn new(name: String) -> Self {
        Self {
            name,
            processor: None,
            filters: Vec::new(),
            should_have_result: false,
        }
    }

    pub fn append(mut self, filters: Vec<Box<dyn Filter>>) -> Self {
        self.filters.extend(filters);
        self
    }

    pub fn should_have_result(mut self) -> Self {
        self.should_have_result = true;
        self
    }

    pub fn process(&self, context: &mut Box<dyn Context>) {
        for filter in &self.filters {
            if context.is_exiting() {
                break;
            }
            filter.process(context);
        }
    }
}

pub struct Processor {
    options: Options,
    pipes: HashMap<String, Box<Pipe>>,
}

impl Processor {
    pub fn new(options: Option<Options>) -> Self {
        Self {
            options: options.unwrap_or_default(),
            pipes: HashMap::new(),
        }
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    pub fn pipe(&mut self, name: &str, pipe: Box<Pipe>) {
        self.pipes.insert(name.to_string(), pipe);
    }

    pub fn get_pipe(&self, name: &str) -> Option<&Box<Pipe>> {
        self.pipes.get(name)
    }

    pub fn process(&self, mut context: Box<dyn Context>) -> Option<Value> {
        context.set_options(self.options.clone());

        let pipe_name = context.pipe();
        if let Some(pipe) = self.pipes.get(pipe_name) {
            pipe.process(&mut context);
        }

        if context.has_result() {
            context.get_result().cloned()
        } else {
            None
        }
    }
}