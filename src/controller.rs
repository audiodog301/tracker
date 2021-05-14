use crate::message::Message;
use tuix::*;

pub struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,

    knob_one: Entity,
    knob_two: Entity,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        Controller {
            command_sender,

            knob_one: Entity::null(),
            knob_two: Entity::null(),
        }
    }
}

impl BuildHandler for Controller {
    type Ret = Entity;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let row = HBox::new().build(state, entity, |builder| {
            builder.set_justify_content(JustifyContent::SpaceEvenly)
        });

        self.knob_one = ValueKnob::new("Amplitude", 1.0, 0.0, 1.0).build(state, row, |builder| {
            builder.set_width(Length::Pixels(50.0))
        });

        self.knob_two =
            ValueKnob::new("Frequency", 440.0, 0.0, 2000.0).build(state, row, |builder| {
                builder.set_width(Length::Pixels(50.0))
            });

        state.focused = entity;

        entity
    }
}

impl EventHandler for Controller {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::KeyDown(code, _) => {
                    if *code == Code::KeyZ {
                        self.command_sender.send(Message::ValueThree(1.0)).unwrap();
                    }
                }

                WindowEvent::KeyUp(code, _) => {
                    if *code == Code::KeyZ {
                        self.command_sender.send(Message::ValueThree(0.0)).unwrap();
                    }
                }

                _ => {}
            }
        }

        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    if event.target == self.knob_one {
                        self.command_sender.send(Message::ValueOne(*val)).unwrap();
                    }

                    if event.target == self.knob_two {
                        self.command_sender.send(Message::ValueTwo(*val)).unwrap();
                    }
                }

                _ => {}
            }
        }
    }
}
