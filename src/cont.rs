use std::cell::RefCell;
use std::rc::Rc;

use super::Env;

type Link = Rc<RefCell<Cont>>;
type OptLink = Option<Link>;

#[derive(Clone, Default)]
pub struct Cont {
    cont: OptLink,
    envt: Rc<Env>,
}

impl Cont {
    pub fn into_rc(self) -> Link {
        Rc::new(RefCell::new(self))
    }

    pub fn from(parent: &Link) -> Self {
        let envt = parent.borrow().envt.clone();

        Self {
            cont: Some(parent.clone()),
            envt,
        }
    }

    pub fn parent(&self) -> OptLink {
        self.cont.clone()
    }

    pub fn set_env(&mut self, envt: Rc<Env>) {
        self.envt = envt;
    }

    pub fn env(&self) -> Rc<Env> {
        self.envt.clone()
    }

    pub fn push(&mut self) {
        self.envt = Env::new(Some(self.envt.clone())).into_rc();
    }

    pub fn pop(&mut self) {
        self.envt = self.envt.parent().unwrap_or_default();
    }
}
