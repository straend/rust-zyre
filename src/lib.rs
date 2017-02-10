extern crate zyre_sys;

use std::error;
use std::ffi::{ CStr, CString };
use std::fmt;
use std::result;
use zyre_sys::{ zmsg_t, zyre_t };

pub type Result<T> = result::Result<T, Error>;

pub struct Error {
  message: String,
}

impl Error {
  fn new(message:&str) -> Error {
    Error {
      message:message.to_owned(),
    }
  }
}

impl error::Error for Error {
  fn description(&self) -> &str {
    &self.message
  }
}

impl fmt::Debug for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{}", &self.message)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{}", &self.message)
  }
}

impl std::convert::From<std::ffi::NulError> for Error {
  fn from(other:std::ffi::NulError) -> Error {
    Error {
      message: {
        use std::error::Error;
        other.description().to_owned()
      },
    }
  }
}

impl std::convert::From<std::str::Utf8Error> for Error {
  fn from(other:std::str::Utf8Error) -> Error {
    Error {
      message: {
        use std::error::Error;
        other.description().to_owned()
      },
    }
  }
}

pub struct Zyre {
  sys: *mut zyre_t,
}

impl Zyre {
  pub fn new(name: &str) -> Result<Zyre> {
    unsafe {
      Ok(Zyre {
        sys: zyre_sys::zyre_new(CString::new(name)?.as_ptr()),
      })
    }
  }

  pub fn destroy(&mut self) {
    unsafe {
      zyre_sys::zyre_destroy(&mut self.sys);
    }
  }

  pub fn uuid(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_uuid(self.sys)).to_str()?)
    }
  }

  pub fn name(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_name(self.sys)).to_str()?)
    }
  }

  pub fn start(&self) -> Result<()> {
    unsafe {
      let rc = zyre_sys::zyre_start(self.sys);
      if rc != 0 {
        // TODO(schoon) - Get the reason from Zyre.
        Err(Error::new("Failed to start Zyre node."))
      } else {
        Ok(())
      }
    }
  }

  pub fn stop(&self) {
    unsafe {
      zyre_sys::zyre_stop(self.sys);
    }
  }

  pub fn join(&self, group:&str) -> Result<()> {
    unsafe {
      let rc = zyre_sys::zyre_join(self.sys, CString::new(group)?.as_ptr());
      if rc != 0 {
        // TODO(schoon) - Get the reason from Zyre.
        Err(Error::new("Failed to join Zyre group."))
      } else {
        Ok(())
      }
    }
  }

  pub fn leave(&self, group:&str) -> Result<()> {
    unsafe {
      let rc = zyre_sys::zyre_leave(self.sys, CString::new(group)?.as_ptr());
      if rc != 0 {
        // TODO(schoon) - Get the reason from Zyre.
        Err(Error::new("Failed to leave Zyre group."))
      } else {
        Ok(())
      }
    }
  }

  pub fn read_event(&self) -> Result<Event> {
    unsafe {
      let event = zyre_sys::zyre_event_new(self.sys);

      if event.is_null() {
        Err(Error::new("Interrupted read_event."))
      } else {
        Ok(Event::new(event))
      }
    }
  }

  pub fn whisper(&self, peer:&str, mut msg:Message) -> Result<()> {
    unsafe {
      zyre_sys::zyre_whisper(self.sys, CString::new(peer)?.as_ptr(), &mut msg.unwrap());
    }

    Ok(())
  }

  pub fn shout(&self, group:&str, mut msg:Message) -> Result<()> {
    unsafe {
      zyre_sys::zyre_shout(self.sys, CString::new(group)?.as_ptr(), &mut msg.unwrap());
    }

    Ok(())
  }
}

impl Drop for Zyre {
  fn drop(&mut self) {
    self.destroy();
  }
}

pub struct Event {
  sys: *mut zyre_sys::zyre_event_t,
  msg: Message,
}

impl Event {
  fn new(event:*mut zyre_sys::zyre_event_t) -> Event {
    Event {
      sys: event,
      msg: unsafe { Message::from_ptr(zyre_sys::zyre_event_get_msg(event)) }
    }
  }

  pub fn destroy(&mut self) {
    unsafe {
      zyre_sys::zyre_event_destroy(&mut self.sys);
    }
  }

  pub fn event_type(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_event_type(self.sys)).to_str()?)
    }
  }

  pub fn peer_uuid(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_event_peer_uuid(self.sys)).to_str()?)
    }
  }

  pub fn peer_name(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_event_peer_name(self.sys)).to_str()?)
    }
  }

  pub fn peer_addr(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_event_peer_addr(self.sys)).to_str()?)
    }
  }

  pub fn group(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zyre_event_group(self.sys)).to_str()?)
    }
  }

  pub fn message(&self) -> &Message {
    &self.msg
  }
}

impl Drop for Event {
  fn drop(&mut self) {
    self.destroy();
  }
}

pub struct Message {
  sys: *mut zmsg_t,
}

impl Message {
  pub fn new() -> Message {
    Message {
      sys:unsafe { zyre_sys::zmsg_new() },
    }
  }

  pub fn from_frames(frames:Vec<&str>) -> Result<Message> {
    let msg = Message::new();

    for frame in frames {
      msg.push(frame)?;
    }

    Ok(msg)
  }

  fn from_ptr(sys:*mut zmsg_t) -> Message {
    Message {
      sys:sys,
    }
  }

  pub fn destroy(&mut self) {
    unsafe {
      zyre_sys::zmsg_destroy(&mut self.sys);
    }
  }

  fn unwrap(&mut self) -> *mut zmsg_t {
    let temp = self.sys;
    self.sys = 0 as *mut _;
    temp
  }

  pub fn size(&self) -> usize {
    unsafe {
      zyre_sys::zmsg_size(self.sys)
    }
  }

  pub fn push(&self, frame:&str) -> Result<()> {
    unsafe {
      zyre_sys::zmsg_pushstr(self.sys, CString::new(frame)?.as_ptr());
    }

    Ok(())
  }

  pub fn pop(&self) -> Result<&str> {
    unsafe {
      Ok(CStr::from_ptr(zyre_sys::zmsg_popstr(self.sys)).to_str()?)
    }
  }

  pub fn collect(&self) -> Result<Vec<&str>> {
    let mut frames = Vec::with_capacity(self.size());

    for _ in 0..self.size() {
      frames.push(unsafe {
        CStr::from_ptr(zyre_sys::zmsg_popstr(self.sys)).to_str()?
      });
    }

    Ok(frames)
  }
}

impl Drop for Message {
  fn drop(&mut self) {
    self.destroy();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_destroy() {
    let mut zyre = Zyre::new("test").ok().unwrap();
    zyre.destroy();
  }

  #[test]
  fn double_destroy() {
    let mut zyre = Zyre::new("test").ok().unwrap();
    zyre.destroy();
    zyre.destroy();
  }

  fn acquire_context<F>(test_fn:F) where F:Fn(&Zyre) {
    let mut zyre = Zyre::new("test").ok().unwrap();

    test_fn(&zyre);

    zyre.destroy();
  }

  #[test]
  fn uuid_length() {
    acquire_context(|zyre:&Zyre| {
      assert_eq!(zyre.uuid().unwrap().len(), 32);
    });
  }

  #[test]
  fn name_value() {
    acquire_context(|zyre:&Zyre| {
      assert_eq!(zyre.name().unwrap(), "test");
    });
  }

  #[test]
  fn start_stop() {
    acquire_context(|zyre:&Zyre| {
      zyre.start().ok();
      zyre.stop();
    });
  }

  fn acquire_started_context<F>(test_fn:F) where F:Fn(&Zyre) {
    acquire_context(|zyre:&Zyre| {
      zyre.start().ok();

      test_fn(&zyre);

      zyre.stop();
    });
  }

  #[test]
  fn join_leave() {
    acquire_started_context(|zyre:&Zyre| {
      zyre.join("GLOBAL").ok();
      zyre.leave("GLOBAL").ok();
    });
  }

  #[test]
  fn read_event() {
    acquire_started_context(|zyre:&Zyre| {
      zyre.read_event().ok();
    });
  }

  #[test]
  fn event_read_destroy() {
    acquire_started_context(|zyre:&Zyre| {
      let mut event = zyre.read_event().unwrap();
      event.destroy();
    });
  }

  #[test]
  fn event_double_destroy() {
    acquire_started_context(|zyre:&Zyre| {
      let mut event = zyre.read_event().unwrap();
      event.destroy();
      event.destroy();
    });
  }

  #[test]
  fn message_new_destroy() {
    let mut message = Message::new();
    message.destroy();
  }

  #[test]
  fn message_double_destroy() {
    let mut message = Message::new();
    message.destroy();
    message.destroy();
  }
}
