use std::fmt::Write;
use std::mem::replace;

use parsley::prelude::*;
use stdweb::*;
use yew::prelude::*;

pub struct Terminal {
    cmd_history: Vec<String>,
    cmd_idx: usize,
    cmd_tmp: Option<String>,
    context: Context,
    history: String,
    value: String,
}

pub enum Msg {
    GotInput(String),
    KeyUp(String),
    Clicked,
}

impl Component for Terminal {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Terminal {
            cmd_history: Vec::new(),
            cmd_idx: 0,
            cmd_tmp: None,
            context: Context::base().capturing(),
            history: String::with_capacity(99999),
            value: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let hist_len = self.cmd_history.len();
        let at_end_of_hist = self.cmd_idx == hist_len;

        match msg {
            Msg::Clicked => {
                js! {
                    document.getElementById("theTerminalInput").focus();
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
                let evaled = run_in(&self.value, &mut self.context);
                // print side effects
                let side_effects = self.context.get_output().unwrap_or_else(String::new);
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
                self.cmd_history
                    .push(replace(&mut self.value, String::new()));
                self.cmd_idx = self.cmd_history.len();
                true
            }
            Msg::GotInput(s) => {
                self.value = s;
                true
            }
            _ => false,
        }
    }
}

impl Renderable<Terminal> for Terminal {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="Terminal", onclick=|_| Msg::Clicked, >
                <div class="History", >
                    { &self.history }
                </div>
                <div class="InputLine", >
                    { "> " }
                    <input
                        id="theTerminalInput",
                        placeholder="Enter an expression...",
                        oninput=|e| Msg::GotInput(e.value),
                        onkeyup=|e| Msg::KeyUp(e.code()),
                        value=&self.value,
                    />
                </div>
            </div>
        }
    }
}
