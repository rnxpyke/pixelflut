use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::sync::Mutex;

use anyhow::{Error, Result};
use tokio::io::AsyncRead;
use crate::net_protocol::{Request, Response};

use crate::state_encoding;

use super::traits::*;
use super::*;

/// A pixmap implementation that proxies through to another pixelflut server.
///
/// It is implemented on generic [`Read`] and [`Write`] trait requirements but these are expected
/// to be a communication channel to something that speaks the pixelflut protocol.
#[derive(Debug)]
pub struct RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    stream: Mutex<(BufReader<I>, BufWriter<F>)>,
    width: usize,
    height: usize,
}

impl<I, F> RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    /// Create a new instance by using the given *reader* and *writer* implementations as a
    /// communication channel.
    ///
    /// The resulting instance will write pixelflut [`OldFrame`]s into *writer* and expects
    /// corresponding [`OldFrame`] responses by reading from *reader*.
    /// An example is to use a TCP socket connected to another pixelflut server and pass
    /// the [`TcpStream`](std::net::TcpStream) (after calling [`try_clone`](std::net::TcpStream::try_clone)) as both
    /// *reader* and *writer*.
    pub fn new(reader: I, writer: F) -> Result<Self> {
        let mut instance = Self {
            stream: Mutex::new((BufReader::new(reader), BufWriter::new(writer))),
            width: 0,
            height: 0,
        };
        instance.fetch_size()?;

        Ok(instance)
    }

    /// send a pixelflut request and wait until a response has been received
    fn send_and_receive(&self, request: Request) -> Result<Response> {
        let mut lock = self.stream.lock().unwrap();

        // send request
        debug!("Sending '{}'", request);
        lock.1
            .write_all(&mut OldFrame::new_from_string(request.to_string()).encode())?;
        lock.1.flush()?;

        // receive response
        // TODO properly use framing instead of just calling read_line
        let mut response = String::new();
        lock.0.read_line(&mut response)?;

        // parse response
        let response = Response::from_str(&response.trim_end_matches('\n'))?;
        Ok(response)
    }

    fn fetch_size(&mut self) -> Result<()> {
        let response = self.send_and_receive(Request::GetSize)?;

        match response {
            Response::Size(width, height) => {
                self.width = width;
                self.height = height;

                Ok(())
            }
            _ => Err(Error::msg(format!(
                "invalid response '{}' for SIZE request",
                response
            ))),
        }
    }
}

impl<I, F> PixmapBase for RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    fn get_size(&self) -> Result<(usize, usize)> {
        Ok((self.width, self.height))
    }
}

impl<I, F> PixmapRead for RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        self.send_and_receive(Request::PxGet(x, y))
            .and_then(|response| match response {
                Response::Px(x2, y2, color) => {
                    if x != x2 || y != y2 {
                        Err(Error::msg(format!(
                            "received color for coordinates {},{} even though {},{} was requested",
                            x, y, x2, y2
                        )))
                    } else {
                        Ok(color)
                    }
                }
                _ => Err(Error::msg(format!(
                    "invalid response '{}' for PX request",
                    response
                ))),
            })
    }
}

impl<I, F> PixmapWrite for RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()> {
        let mut lock = self.stream.lock().unwrap();
        let request = Request::PxSet(x, y, color);

        // send request without waiting for response
        debug!(target: LOG_TARGET, "Sending '{}'", request);
        lock.1
            .write_all(&OldFrame::new_from_string(request.to_string()).encode())?;
        lock.1.flush()?;

        Ok(())
    }
}

impl<I, F> PixmapRawRead for RemotePixmap<I, F>
where
    I: Read,
    F: Write,
{
    fn get_raw_data(&self) -> Result<Vec<Color>> {
        self.send_and_receive(Request::State(StateEncodingAlgorithm::Rgb64))
            .and_then(|response| match response {
                Response::State(StateEncodingAlgorithm::Rgb64, data) => state_encoding::rgb64::decode(data),
                _ => Err(Error::msg("invalid response for STATE request")),
            })
    }
}

#[cfg(test)]
mod test {}
