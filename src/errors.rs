#[derive(Debug)]
pub enum MyError {
    IO(std::io::Error),
    JSON(serde_json::error::Error),
    Image(image::ImageError),
}

impl From<std::io::Error> for MyError {
    fn from(error: std::io::Error) -> MyError {
        MyError::IO(error)
    }
}

impl From<serde_json::error::Error> for MyError {
    fn from(error: serde_json::error::Error) -> MyError {
        MyError::JSON(error)
    }
}

impl From<image::ImageError> for MyError {
    fn from(error: image::ImageError) -> MyError {
        MyError::Image(error)
    }
}
