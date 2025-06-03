use std::error::Error as StdError;
use std::marker::PhantomData;

/// Error reporter is any object that can be used to collect errors during validation by
/// [`Validation`] or [`ContextualValidation`] traits.
///
/// This is usually done directly by [`VecReporter`], or using a type wrapper constructed
/// based on [`ErrorReporter::wrap`]. However, in most cases, users will just use `validate`
/// methods that use the default vector reporter without caring too much about the underlying
/// implementation.
pub trait ErrorReporter<E: StdError>: Sized {
    /// Report an error into this [`ErrorReporter`].
    fn report<E2: Into<E>>(&mut self, error: E2);

    /// Wrap a mutable reference to this [`ErrorReporter`] into a [`ReporterWrapper`]
    /// which automatically performs type conversions from `E2`.
    fn wrap<E2: StdError + Into<E>>(&mut self) -> ReporterWrapper<E2, E, Self> {
        ReporterWrapper {
            inner: self,
            _e1: PhantomData,
            _e2: PhantomData,
        }
    }
}

/// A simple [`ErrorReporter`] implementation that collects all errors into a vector.
pub struct VecReporter<E: StdError> {
    errors: Vec<E>,
}

/// A simple [`ErrorReporter`] implementation that defers to an internal [`ErrorReporter`]
/// by performing type conversion from `E1` into `E2` using `Into`.
pub struct ReporterWrapper<'a, E1: StdError + Into<E2>, E2: StdError, W: ErrorReporter<E2>> {
    inner: &'a mut W,
    _e1: PhantomData<E1>,
    _e2: PhantomData<E2>,
}

impl<'a, E1: StdError + Into<E2>, E2: StdError, W: ErrorReporter<E2>> ErrorReporter<E1>
    for ReporterWrapper<'a, E1, E2, W>
{
    fn report<X: Into<E1>>(&mut self, error: X) {
        self.inner.report(error.into());
    }
}

impl<E: StdError> ErrorReporter<E> for VecReporter<E> {
    fn report<X: Into<E>>(&mut self, error: X) {
        self.errors.push(error.into());
    }
}

/// Contextual validation trait is implemented by objects that can only be validated against a
/// certain context. Usually, this happens when a certain type needs extra information to
/// infer that all of its invariants are satisfied.
///
/// Typically, the context is assumed to be immutable during validation.
///
/// Sometimes, you want to implement multiple validation scenarios (for example, a scenario that
/// only finds errors and a scenario that finds warnings). In that case, you can wrap the `Context`
/// into an additional wrapper type (e.g. `CheckErrors<Context>` and `CheckWarnings<Context>`).
/// Then, you can provide several implementations of `ContextualValidation` that
/// are parametrized by the context type.
///
/// Compared to `From` and `Into` traits, validation generally does not terminate when the first
/// error is found. Instead, it collects all errors into a provided [ErrorReporter].
pub trait ContextualValidation<Context> {
    /// The type of error that can be thrown during validation.
    type Error: StdError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &Context, reporter: &mut R);

    fn validate(&self, context: &Context) -> Result<(), Vec<Self::Error>> {
        let mut reporter = VecReporter { errors: vec![] };
        self.validate_all(context, &mut reporter);
        if reporter.errors.is_empty() {
            Ok(())
        } else {
            Err(reporter.errors)
        }
    }
}

/// Validation trait is implemented by objects that can be validated.
///
/// Each validation process reports errors using a provided [ErrorReporter]. Compared to
/// traits like `From` and `Into`, validation can typically produce more than one error.
///
/// If you need to validate objects whose behavior (or validity) depends on some additional
/// data, consider implementing [ContextualValidation].
pub trait Validation {
    type Error: StdError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, reporter: &mut R);

    fn validate(&self) -> Result<(), Vec<Self::Error>> {
        let mut reporter = VecReporter { errors: vec![] };
        self.validate_all(&mut reporter);
        if reporter.errors.is_empty() {
            Ok(())
        } else {
            Err(reporter.errors)
        }
    }
}
