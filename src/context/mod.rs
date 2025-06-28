pub mod diff;

pub use diff::DiffContext;

use crate::types::Options;
use serde_json::Value;

pub trait FilterContext {
    type Result;

    fn set_result(&mut self, result: Self::Result) -> &mut Self;
    fn get_result(&self) -> Option<&Self::Result>;
    fn exit(&mut self) -> &mut Self;
    fn is_exiting(&self) -> bool;

    // fn inner_context_clone(&self) -> Rc<RefCell<Self>>;
    fn inner_data(&mut self) -> &mut MyContext<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct MyContext<FC: FilterContext> {
    pipe: String,
    result: Option<FC::Result>,
    exiting: bool,
    // parent: Option<Rc<FC>>,
    child_name: Option<String>,
    // root: Option<Rc<MyContext<FC>>>,
    pub options: Option<Options>,
    // pub children: Vec<Rc<RefCell<FC>>>,
    // pub next_after_children: Option<Rc<RefCell<FC>>>,
    // pub next: Option<Rc<RefCell<FC>>>,
}

impl<FC: FilterContext> MyContext<FC> {
    pub fn new(pipe: String) -> Self {
        Self {
            pipe,
            result: None,
            exiting: false,
            // parent: None,
            child_name: None,
            // root: None,
            options: None,
            // children: vec![],
            // next_after_children: None,
            // next: None,
        }
    }

    pub fn is_exiting(&self) -> bool {
        // let arena = Arena::new();
        // let out = arena.alloc(Some(5));

        self.exiting
    }

    pub fn set_result(&mut self, result: FC::Result) {
        self.result = Some(result);
    }

    pub fn exit(&mut self) {
        self.exiting = true;
    }
}

// pub fn add_child_to_context<FC: FilterContext>(
//     context: Rc<RefCell<FC>>,
//     child: Rc<RefCell<FC>>,
//     name: Option<String>,
// ) {
//     let mut child_mut = child.borrow_mut();
//     let child_mut = child_mut.inner_data();
//     let mut context_mut = context.borrow_mut();
//     let context_mut = context_mut.inner_data();

//     if let Some(name) = name {
//         child_mut.child_name = Some(name);
//     }
//     // child.root = Some(self.root.clone().unwrap());

//     if let (None, Some(options)) = (&child_mut.options, &context_mut.options) {
//         child_mut.options = Some(options.clone());
//     }

//     if context_mut.children.is_empty() {
//         context_mut.children.push(child.clone());
//         if context_mut.next.is_some() {
//             context_mut.next_after_children = context_mut.next.take();
//         }
//         context_mut.next = Some(child.clone());
//     } else {
//         let last_child = context_mut.children.last_mut().unwrap();
//         last_child.borrow_mut().inner_data().next = Some(child.clone());
//         context_mut.children.push(child.clone());
//     }

//     child.borrow_mut().inner_data().next = Some(context.clone());
// }

pub trait ContextOld {
    fn pipe(&self) -> &str;
    fn set_result(&mut self, result: Value);
    fn exit(&mut self);
    fn get_options(&self) -> &Options;
    fn set_options(&mut self, options: Options);
    fn get_result(&self) -> Option<&Value>;
    fn has_result(&self) -> bool;
    fn is_exiting(&self) -> bool;
}

// // Implement ContextOld for DiffContext
// impl ContextOld for DiffContext {
//     fn pipe(&self) -> &str {
//         &self.pipe
//     }

//     fn set_result(&mut self, result: Value) {
//         // Convert Value to Delta if possible, or handle appropriately
//         // For now, we'll just set a placeholder
//         self.has_result = true;
//     }

//     fn exit(&mut self) {
//         self.exiting = true;
//     }

//     fn get_options(&self) -> &Options {
//         &self.options
//     }

//     fn set_options(&mut self, options: Options) {
//         self.options = options;
//     }

//     fn get_result(&self) -> Option<&Value> {
//         // Convert Delta to Value if needed
//         None
//     }

//     fn has_result(&self) -> bool {
//         self.has_result
//     }

//     fn is_exiting(&self) -> bool {
//         self.exiting
//     }
// }
