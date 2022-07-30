use std::fmt::Write;
use std::mem::take;

use yew::prelude::*;

pub struct Terminal {
    cmd_history: Vec<String>,
    cmd_idx: usize,
    cmd_tmp: Option<String>,
    context: parsley::Context,
    history: String,
    value: String,
    input_ref: NodeRef,
}

pub enum Msg {
    GotInput,
    KeyUp(String),
    Clicked,
}

impl Component for Terminal {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Terminal {
            cmd_history: Vec::new(),
            cmd_idx: 0,
            cmd_tmp: None,
            context: parsley::Context::base().capturing(),
            history: String::with_capacity(99999),
            value: String::new(),
            input_ref: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let hist_len = self.cmd_history.len();
        let at_end_of_hist = self.cmd_idx == hist_len;

        match msg {
            Msg::Clicked => {
                if let Some(input) = self.input_ref.cast::<web_sys::HtmlInputElement>() {
                    let _dont_care_if_it_fails = input.focus();
                }
                false
            }
            Msg::KeyUp(ref s) if s == "ArrowUp" && self.cmd_idx != 0 => {
                if at_end_of_hist {
                    self.cmd_tmp = Some(self.value.clone());
                }
                self.cmd_idx -= 1;
                self.value = self.cmd_history[self.cmd_idx].clone();
                true
            }
            Msg::KeyUp(ref s) if s == "ArrowDown" && !at_end_of_hist => {
                self.cmd_idx += 1;
                if self.cmd_idx == self.cmd_history.len() {
                    if let Some(c) = &self.cmd_tmp {
                        self.value = c.clone();
                        self.cmd_tmp = None;
                    } else {
                        self.value = String::new();
                    }
                } else {
                    self.value = self.cmd_history[self.cmd_idx].clone();
                }
                true
            }
            Msg::KeyUp(ref s) if s == "Enter" && !self.value.is_empty() => {
                // show command in history
                writeln!(self.history, "> {}", self.value).unwrap();
                // evaluate
                let evaled = self.context.run(&self.value);
                // print side effects
                let side_effects = self.context.get_output().unwrap_or_default();
                if !side_effects.is_empty() {
                    self.history.push_str(&side_effects);
                }
                self.context.capture();
                // show actual output
                match evaled {
                    Ok(result) => {
                        // print result, if it's not empty
                        let res = format!("{}", result);
                        if !res.is_empty() {
                            writeln!(self.history, "{}", res).unwrap();
                        }
                    }
                    Err(error) => {
                        // save error
                        writeln!(self.history, "{}", error).unwrap();
                    }
                }
                // save command and create buffer for new one
                self.cmd_tmp = None;
                self.cmd_history.push(take(&mut self.value));
                self.cmd_idx = self.cmd_history.len();
                true
            }
            Msg::GotInput => {
                if let Some(input) = self.input_ref.cast::<web_sys::HtmlInputElement>() {
                    self.value = input.value();
                }
                true
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="Terminal" onclick={ctx.link().callback(|_| Msg::Clicked)} >
                <div class="History" >
                    { &self.history }
                </div>
                <div class="InputLine" >
                    { "> " }
                    <input
                    	ref={self.input_ref.clone()}
                        placeholder="Enter an expression..."
                        oninput={ctx.link().callback(|_| Msg::GotInput)}
                        onkeyup={ctx.link().callback(|e: KeyboardEvent| Msg::KeyUp(e.code()))}
                        value={ self.value.clone() }
                    />
                </div>
            </div>
        }
    }
}
