use crate::context::FilterContext;
use crate::types::Options;

pub trait Filter<C, TResult> {
    fn filter_name(&self) -> &str;
    fn process(&self, context: &mut C, new_children_context: &mut Vec<(String, C)>);

    fn post_process(&self, context: &mut C, new_children_context: &mut Vec<(String, C)>);
}

pub struct Pipe<C: FilterContext, TResult> {
    name: String,
    // processor: Option<Box<Processor>>,
    filters: Vec<Box<dyn Filter<C, TResult>>>,
    should_have_result: bool,
}

impl<C: FilterContext, TResult> Pipe<C, TResult> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            // processor: None,
            filters: Vec::new(),
            should_have_result: false,
        }
    }

    pub fn append(mut self, filter: Box<dyn Filter<C, TResult>>) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn should_have_result(mut self) -> Self {
        self.should_have_result = true;
        self
    }

    pub fn process(&self, context: &mut C, new_children_context: &mut Vec<(String, C)>) {
        log::trace!("process: {}", self.name);

        for filter in &self.filters {
            log::trace!("filter: {}", filter.filter_name());
            filter.process(context, new_children_context);

            if context.is_exiting() {
                break;
            }
        }
    }

    pub fn post_process(&self, context: &mut C, children_context: &mut Vec<(String, C)>) {
        for filter in &self.filters {
            filter.post_process(context, children_context);
        }
    }
}

pub enum PipeType {
    Diff,
    // Patch,
    // Reverse,
}

pub struct Processor {
    options: Options,
    // pipes: HashMap<String, Box<Pipe>>,
    // diff_pipe: Pipe<DiffContext, Value>,
}

impl Processor {
    pub fn new(options: Option<Options>) -> Self {
        Self {
            options: options.unwrap_or_default(),
            // pipes: HashMap::new(),
        }
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    // pub fn pipe(&mut self, name: &str, pipe: Box<Pipe>) {
    //     self.pipes.insert(name.to_string(), pipe);
    // }

    // pub fn get_pipe(&self, pipe_type: PipeType) -> Option<&Box<Pipe>> {
    //     match pipe_type {
    //         PipeType::Diff => Some(&self.diff_pipe),
    //         // PipeType::Patch => Some(&self.patch_pipe),
    //         // PipeType::Reverse => Some(&self.reverse_pipe),
    //     }
    // }

    pub fn process<TContext: FilterContext>(
        &self,
        context: &mut TContext,
        pipeline: &mut Pipe<TContext, TContext::Result>,
    )
    // -> Option<TContext::Result>
    {
        // context.set_options(self.options.clone());

        // Create a simplified options clone without function pointers
        let simplified_options = Options {
            // object_hash: None, // Cannot clone function pointers
            match_by_position: self.options.match_by_position,
            arrays: self.options.arrays.clone(),
            text_diff: self.options.text_diff.clone(),
            // property_filter: None, // Cannot clone function pointers
            clone_diff_values: self.options.clone_diff_values,
            omit_removed_values: self.options.omit_removed_values,
        };

        // context.set_options(simplified_options);
        let context = context;
        let inner_data = context.inner_data();
        inner_data.options = Some(self.options.clone());

        // let pipe_name = context.pipe();

        // let mut last_pipe = None;
        // let mut next_pipe = Some(pipeline);

        {
            // if inner_data.next_after_children.is_some() {
            //     inner_data.next = inner_data.next_after_children.take();
            // }

            let mut new_children_context = vec![];

            pipeline.process(context, &mut new_children_context);

            if new_children_context.is_empty() {
                // continue to process the next queue
                // continue;
            }

            // we need to push the current context to a stack

            for (key, child) in &mut new_children_context {
                // recursively process the child context
                // it's better to use a stack here, as, if the json is too deep, it will cause a stack overflow
                self.process(child, pipeline);
            }

            pipeline.post_process(context, &mut new_children_context);

            // log::error!("context.name: {}", context.child_name());

            // for (key, child) in new_children_context {
            //     context.push(child, key);
            // }

            // last_pipe = next_pipe.take();

            // if inner_data.next.is_some() {
            //     context = inner_data.next.take().unwrap();
            //     inner_data.next = inner_data.next.take();
            // }
        }

        // context.get_result().cloned()

        // let next_pipe =

        // todo!();

        // use crate::context::diff::DiffContext;

        // let pipe: Pipe<DiffContext, Value> = todo!();

        // // let pipe = self.get_pipe(pipe_name).expect("Pipe not found");
        // pipe.process(&mut context);

        // if context.has_result() {
        //     context.get_result().cloned()
        // } else {
        //     None
        // }
        // None
    }
}
