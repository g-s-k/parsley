use std::cell::RefCell;
use std::rc::Rc;

use super::Env;

type Link = Rc<RefCell<Cont>>;
type OptLink = Option<Link>;

#[derive(Clone, Default)]
pub struct Cont {
    cont: OptLink,
    clos: Option<Rc<Env>>,
    envt: Rc<Env>,
}

impl Cont {
    pub fn into_rc(self) -> Link {
        Rc::new(RefCell::new(self))
    }

    pub fn new(parent: OptLink, env: Rc<Env>) -> Self {
        Self {
            cont: parent,
            clos: None,
            envt: env,
        }
    }

    pub fn parent(&self) -> OptLink {
        self.cont.clone()
    }

    pub fn use_cls(&mut self, closure: Option<Rc<Env>>) {
        self.clos = closure;
    }

    pub fn cls(&self) -> Option<Rc<Env>> {
        self.clos.clone()
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
