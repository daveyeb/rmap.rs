use axum::response::{IntoResponse, Redirect};

#[derive(Debug)]
pub enum Error {
    NotFound,
    Unauthorized,
    InternalServerError,
    BadRequest,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let response = match self {
            Error::Unauthorized => (Redirect::to("/")).into_response(),
            Error::NotFound => (Redirect::to("/badpage")).into_response(),
            Error::InternalServerError => (Redirect::to("/err")).into_response(),
            Error::BadRequest => (Redirect::to("/badrequest")).into_response(),
        };

        response
    }
}
