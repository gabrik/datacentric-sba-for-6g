#![allow(
    missing_docs,
    trivial_casts,
    unused_variables,
    unused_mut,
    unused_imports,
    unused_extern_crates,
    non_camel_case_types
)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::blacklisted_name)]

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "/nsmf-pdusession/v1";
pub const API_VERSION: &str = "1.2.2";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ReleasePduSessionResponse {
    /// successful release of a PDU session with content in the response
    SuccessfulReleaseOfAPDUSessionWithContentInTheResponse(models::ReleasedData),
    /// successful release of a PDU session
    SuccessfulReleaseOfAPDUSession,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum RetrievePduSessionResponse {
    /// successful information retrieval
    SuccessfulInformationRetrieval(models::RetrievedData),
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Gateway Timeout
    GatewayTimeout(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum TransferMoDataResponse {
    /// successful transfering of MO data
    SuccessfulTransferingOfMOData,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Unauthorized
    Unauthorized(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UpdatePduSessionResponse {
    /// successful update of a PDU session with content in the response
    SuccessfulUpdateOfAPDUSessionWithContentInTheResponse(models::HsmfUpdatedData),
    /// successful update of a PDU session without content in the response
    SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession(models::HsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_2(models::HsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_3(models::HsmfUpdateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_4(models::HsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_5(models::HsmfUpdateError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ReleaseSmContextResponse {
    /// successful release of a PDU session with content in the response
    SuccessfulReleaseOfAPDUSessionWithContentInTheResponse(models::SmContextReleasedData),
    /// successful release of an SM context without content in the response
    SuccessfulReleaseOfAnSMContextWithoutContentInTheResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum RetrieveSmContextResponse {
    /// successful retrieval of an SM context
    SuccessfulRetrievalOfAnSMContext(models::SmContextRetrievedData),
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Gateway Timeout
    GatewayTimeout(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SendMoDataResponse {
    /// successful sending of MO data
    SuccessfulSendingOfMOData,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Unauthorized
    Unauthorized(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UpdateSmContextResponse {
    /// successful update of an SM context with content in the response
    SuccessfulUpdateOfAnSMContextWithContentInTheResponse(models::SmContextUpdatedData),
    /// successful update of an SM context without content in the response
    SuccessfulUpdateOfAnSMContextWithoutContentInTheResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful update of an SM context - bad request
    UnsuccessfulUpdateOfAnSMContext(models::SmContextUpdateError),
    /// unsuccessful update of an SM context - forbidden
    UnsuccessfulUpdateOfAnSMContext_2(models::SmContextUpdateError),
    /// unsuccessful update of an SM context - not found
    UnsuccessfulUpdateOfAnSMContext_3(models::SmContextUpdateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful update of an SM context - Internal server error
    UnsuccessfulUpdateOfAnSMContext_4(models::SmContextUpdateError),
    /// unsuccessful update of an SM context - Service Unavailable
    UnsuccessfulUpdateOfAnSMContext_5(models::SmContextUpdateError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum PostPduSessionsResponse {
    /// successful creation of a PDU session
    SuccessfulCreationOfAPDUSession {
        body: models::PduSessionCreatedData,
        location: String,
    },
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful creation of a PDU session
    UnsuccessfulCreationOfAPDUSession(models::PduSessionCreateError),
    /// unsuccessful creation of a PDU session
    UnsuccessfulCreationOfAPDUSession_2(models::PduSessionCreateError),
    /// unsuccessful creation of a PDU session
    UnsuccessfulCreationOfAPDUSession_3(models::PduSessionCreateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful creation of a PDU session
    UnsuccessfulCreationOfAPDUSession_4(models::PduSessionCreateError),
    /// unsuccessful creation of a PDU session
    UnsuccessfulCreationOfAPDUSession_5(models::PduSessionCreateError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum PostSmContextsResponse {
    /// successful creation of an SM context
    SuccessfulCreationOfAnSMContext {
        body: models::SmContextCreatedData,
        location: String,
    },
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful creation of an SM context - bad request
    UnsuccessfulCreationOfAnSMContext(models::SmContextCreateError),
    /// unsuccessful creation of an SM context - forbidden
    UnsuccessfulCreationOfAnSMContext_2(models::SmContextCreateError),
    /// unsuccessful creation of an SM context - not found
    UnsuccessfulCreationOfAnSMContext_3(models::SmContextCreateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful creation of an SM context - internal server error
    UnsuccessfulCreationOfAnSMContext_4(models::SmContextCreateError),
    /// unsuccessful creation of an SM context - service unavailable
    UnsuccessfulCreationOfAnSMContext_5(models::SmContextCreateError),
    /// unsuccessful creation of an SM context - gateway timeout
    UnsuccessfulCreationOfAnSMContext_6(models::SmContextCreateError),
    /// Generic Error
    GenericError,
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
        context: &C,
    ) -> Result<ReleasePduSessionResponse, ApiError>;

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
        context: &C,
    ) -> Result<RetrievePduSessionResponse, ApiError>;

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMoDataResponse, ApiError>;

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
        context: &C,
    ) -> Result<UpdatePduSessionResponse, ApiError>;

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
        context: &C,
    ) -> Result<ReleaseSmContextResponse, ApiError>;

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
        context: &C,
    ) -> Result<RetrieveSmContextResponse, ApiError>;

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<SendMoDataResponse, ApiError>;

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
        context: &C,
    ) -> Result<UpdateSmContextResponse, ApiError>;

    /// Create
    async fn post_pdu_sessions(
        &self,
        pdu_session_create_data: models::PduSessionCreateData,
        context: &C,
    ) -> Result<PostPduSessionsResponse, ApiError>;

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<swagger::ByteArray>,
        binary_data_n2_sm_information: Option<swagger::ByteArray>,
        binary_data_n2_sm_information_ext1: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<PostSmContextsResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
    ) -> Result<ReleasePduSessionResponse, ApiError>;

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
    ) -> Result<RetrievePduSessionResponse, ApiError>;

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMoDataResponse, ApiError>;

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
    ) -> Result<UpdatePduSessionResponse, ApiError>;

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
    ) -> Result<ReleaseSmContextResponse, ApiError>;

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
    ) -> Result<RetrieveSmContextResponse, ApiError>;

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
    ) -> Result<SendMoDataResponse, ApiError>;

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
    ) -> Result<UpdateSmContextResponse, ApiError>;

    /// Create
    async fn post_pdu_sessions(
        &self,
        pdu_session_create_data: models::PduSessionCreateData,
    ) -> Result<PostPduSessionsResponse, ApiError>;

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<swagger::ByteArray>,
        binary_data_n2_sm_information: Option<swagger::ByteArray>,
        binary_data_n2_sm_information_ext1: Option<swagger::ByteArray>,
    ) -> Result<PostSmContextsResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
    ) -> Result<ReleasePduSessionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .release_pdu_session(pdu_session_ref, release_data, &context)
            .await
    }

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
    ) -> Result<RetrievePduSessionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .retrieve_pdu_session(pdu_session_ref, retrieve_data, &context)
            .await
    }

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMoDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .transfer_mo_data(pdu_session_ref, json_data, binary_mo_data, &context)
            .await
    }

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
    ) -> Result<UpdatePduSessionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .update_pdu_session(pdu_session_ref, hsmf_update_data, &context)
            .await
    }

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
    ) -> Result<ReleaseSmContextResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .release_sm_context(sm_context_ref, sm_context_release_data, &context)
            .await
    }

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
    ) -> Result<RetrieveSmContextResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .retrieve_sm_context(sm_context_ref, sm_context_retrieve_data, &context)
            .await
    }

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
    ) -> Result<SendMoDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .send_mo_data(sm_context_ref, json_data, binary_mo_data, &context)
            .await
    }

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
    ) -> Result<UpdateSmContextResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .update_sm_context(sm_context_ref, sm_context_update_data, &context)
            .await
    }

    /// Create
    async fn post_pdu_sessions(
        &self,
        pdu_session_create_data: models::PduSessionCreateData,
    ) -> Result<PostPduSessionsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .post_pdu_sessions(pdu_session_create_data, &context)
            .await
    }

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<swagger::ByteArray>,
        binary_data_n2_sm_information: Option<swagger::ByteArray>,
        binary_data_n2_sm_information_ext1: Option<swagger::ByteArray>,
    ) -> Result<PostSmContextsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .post_sm_contexts(
                json_data,
                binary_data_n1_sm_message,
                binary_data_n2_sm_information,
                binary_data_n2_sm_information_ext1,
                &context,
            )
            .await
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum NotifyStatusResponse {
    /// successful notificationof the status change
    SuccessfulNotificationofTheStatusChange,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum NotifyStatusIsfmResponse {
    /// successful notificationof the status change
    SuccessfulNotificationofTheStatusChange,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ModifyPduSessionResponse {
    /// successful update of a PDU session with content in the response
    SuccessfulUpdateOfAPDUSessionWithContentInTheResponse(models::VsmfUpdatedData),
    /// successful update of a PDU session without content in the response
    SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_2(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_3(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_4(models::VsmfUpdateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_5(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_6(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_7(models::VsmfUpdateError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ModifyPduSessionIsmfResponse {
    /// successful update of a PDU session with content in the response
    SuccessfulUpdateOfAPDUSessionWithContentInTheResponse(models::VsmfUpdatedData),
    /// successful update of a PDU session without content in the response
    SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_2(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_3(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_4(models::VsmfUpdateError),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_5(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_6(models::VsmfUpdateError),
    /// unsuccessful update of a PDU session
    UnsuccessfulUpdateOfAPDUSession_7(models::VsmfUpdateError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum TransferMtDataResponse {
    /// successful transfering of MT data
    SuccessfulTransferingOfMTData,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Unauthorized
    Unauthorized(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// unsuccessful delivery of mobile terminated data - gateway timeout
    UnsuccessfulDeliveryOfMobileTerminatedData(models::TransferMtDataError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum TransferMtDataIsmfResponse {
    /// successful transfering of MT data
    SuccessfulTransferingOfMTData,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Unauthorized
    Unauthorized(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
    /// unsuccessful delivery of mobile terminated data - gateway timeout
    UnsuccessfulDeliveryOfMobileTerminatedData(models::TransferMtDataError),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SmContextStatusNotificationPostResponse {
    /// successful notification
    SuccessfulNotification,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ExtProblemDetails),
    /// Forbidden
    Forbidden(models::ExtProblemDetails),
    /// Not Found
    NotFound(models::ExtProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ExtProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ExtProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ExtProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ExtProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ExtProblemDetails),
}

/// Callback API
#[async_trait]
pub trait CallbackApi<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Notify Status
    async fn notify_status(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
        context: &C,
    ) -> Result<NotifyStatusResponse, ApiError>;

    /// Notify Status
    async fn notify_status_isfm(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
        context: &C,
    ) -> Result<NotifyStatusIsfmResponse, ApiError>;

    /// Update (initiated by H-SMF)
    async fn modify_pdu_session(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
        context: &C,
    ) -> Result<ModifyPduSessionResponse, ApiError>;

    /// Update (initiated by SMF)
    async fn modify_pdu_session_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
        context: &C,
    ) -> Result<ModifyPduSessionIsmfResponse, ApiError>;

    /// Transfer MT Data (by H-SMF)
    async fn transfer_mt_data(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMtDataResponse, ApiError>;

    /// Transfer MT Data (by SMF)
    async fn transfer_mt_data_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMtDataIsmfResponse, ApiError>;

    async fn sm_context_status_notification_post(
        &self,
        callback_request_body_sm_context_status_uri: String,
        sm_context_status_notification: models::SmContextStatusNotification,
        context: &C,
    ) -> Result<SmContextStatusNotificationPostResponse, ApiError>;
}

/// Callback API without a `Context`
#[async_trait]
pub trait CallbackApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Notify Status
    async fn notify_status(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
    ) -> Result<NotifyStatusResponse, ApiError>;

    /// Notify Status
    async fn notify_status_isfm(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
    ) -> Result<NotifyStatusIsfmResponse, ApiError>;

    /// Update (initiated by H-SMF)
    async fn modify_pdu_session(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
    ) -> Result<ModifyPduSessionResponse, ApiError>;

    /// Update (initiated by SMF)
    async fn modify_pdu_session_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
    ) -> Result<ModifyPduSessionIsmfResponse, ApiError>;

    /// Transfer MT Data (by H-SMF)
    async fn transfer_mt_data(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMtDataResponse, ApiError>;

    /// Transfer MT Data (by SMF)
    async fn transfer_mt_data_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMtDataIsmfResponse, ApiError>;

    async fn sm_context_status_notification_post(
        &self,
        callback_request_body_sm_context_status_uri: String,
        sm_context_status_notification: models::SmContextStatusNotification,
    ) -> Result<SmContextStatusNotificationPostResponse, ApiError>;
}

pub trait CallbackContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: CallbackApi<C> + Send + Sync, C: Clone + Send + Sync> CallbackContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: CallbackApi<C> + Send + Sync, C: Clone + Send + Sync> CallbackApiNoContext<C>
    for ContextWrapper<T, C>
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Notify Status
    async fn notify_status(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
    ) -> Result<NotifyStatusResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .notify_status(
                callback_request_body_vsmf_pdu_session_uri,
                status_notification,
                &context,
            )
            .await
    }

    /// Notify Status
    async fn notify_status_isfm(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        status_notification: models::StatusNotification,
    ) -> Result<NotifyStatusIsfmResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .notify_status_isfm(
                callback_request_body_ismf_pdu_session_uri,
                status_notification,
                &context,
            )
            .await
    }

    /// Update (initiated by H-SMF)
    async fn modify_pdu_session(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
    ) -> Result<ModifyPduSessionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .modify_pdu_session(
                callback_request_body_vsmf_pdu_session_uri,
                vsmf_update_data,
                &context,
            )
            .await
    }

    /// Update (initiated by SMF)
    async fn modify_pdu_session_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        vsmf_update_data: models::VsmfUpdateData,
    ) -> Result<ModifyPduSessionIsmfResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .modify_pdu_session_ismf(
                callback_request_body_ismf_pdu_session_uri,
                vsmf_update_data,
                &context,
            )
            .await
    }

    /// Transfer MT Data (by H-SMF)
    async fn transfer_mt_data(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMtDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .transfer_mt_data(
                callback_request_body_vsmf_pdu_session_uri,
                json_data,
                binary_mt_data,
                &context,
            )
            .await
    }

    /// Transfer MT Data (by SMF)
    async fn transfer_mt_data_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        json_data: Option<models::TransferMtDataReqData>,
        binary_mt_data: Option<swagger::ByteArray>,
    ) -> Result<TransferMtDataIsmfResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .transfer_mt_data_ismf(
                callback_request_body_ismf_pdu_session_uri,
                json_data,
                binary_mt_data,
                &context,
            )
            .await
    }

    async fn sm_context_status_notification_post(
        &self,
        callback_request_body_sm_context_status_uri: String,
        sm_context_status_notification: models::SmContextStatusNotification,
    ) -> Result<SmContextStatusNotificationPostResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .sm_context_status_notification_post(
                callback_request_body_sm_context_status_uri,
                sm_context_status_notification,
                &context,
            )
            .await
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(any(feature = "client", feature = "server"))]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
