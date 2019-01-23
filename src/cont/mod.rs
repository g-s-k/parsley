use std::rc::Rc;

use super::Env;

pub struct Cont {
    cont: Option<Rc<Cont>>,
    envt: Rc<Env>,
}

impl Default for Cont {
    fn default() -> Self {
        Self {
            cont: None,
            envt: Rc::new(Env::new()),
        }
    }
}

impl Cont {
    pub fn as_link(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn new(parent: Option<Rc<Self>>, env: Rc<Env>) -> Self {
        Self {
            cont: parent,
            envt: env,
        }
    }

    pub fn parent(&self) -> Option<Rc<Self>> {
        self.cont.clone()
    }

    pub fn env(&self) -> Rc<Env> {
        self.envt.clone()
    }
}
