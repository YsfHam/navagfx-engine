use winit::event::WindowEvent;


#[derive(Debug)]
pub enum ApplicationEvent {
    Resized {width: u32, height: u32},
}



impl ApplicationEvent {
    pub fn from_window_event(event: WindowEvent) -> Option<Self> {
        match event {
            WindowEvent::Resized(size) => {
                Some(Self::Resized { width: size.width, height: size.height })
            }
            _ => None
        }
    }
}


pub enum ApplicationSignal {
    Exit,
    Continue,
}