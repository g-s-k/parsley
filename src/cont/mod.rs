use std::rc::Rc;

use super::Env;

type Link = Option<Rc<Cont>>;

#[derive(Clone)]
pub struct Cont {
    cont: Link,
    envt: Rc<Env>,
}

impl Default for Cont {
    fn default() -> Self {
        Self {
            cont: None,
            envt: Rc::new(Env::default()),
        }
    }
}

impl Cont {
    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn new(parent: Link, env: Rc<Env>) -> Self {
        Self {
            cont: parent,
            envt: env,
        }
    }

    pub fn parent(&self) -> Link {
        self.cont.clone()
    }

    pub fn env(&self) -> Rc<Env> {
        self.envt.clone()
    }
}
