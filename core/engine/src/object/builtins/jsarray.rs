//! A Rust API wrapper for Boa's `Array` Builtin ECMAScript Object
use crate::{
    Context, JsResult, JsString, JsValue,
    builtins::Array,
    error::JsNativeError,
    object::{JsFunction, JsObject},
    value::{IntoOrUndefined, TryFromJs},
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsArray` provides a wrapper for Boa's implementation of the JavaScript `Array` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsArray {
    inner: JsObject,
}

impl JsArray {
    /// Create a new empty array.
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let inner = Array::array_create(0, None, context)
            .expect("creating an empty array with the default prototype must not fail");

        Self { inner }
    }

    /// Create an array from a `IntoIterator<Item = JsValue>` convertible object.
    pub fn from_iter<I>(elements: I, context: &mut Context) -> Self
    where
        I: IntoIterator<Item = JsValue>,
    {
        Self {
            inner: Array::create_array_from_list(elements, context),
        }
    }

    /// Create a [`JsArray`] from a [`JsObject`], if the object is not an array throw a `TypeError`.
    ///
    /// This does not clone the fields of the array, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_array() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an Array")
                .into())
        }
    }

    /// Get the length of the array.
    ///
    /// Same as `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: &mut Context) -> JsResult<u64> {
        self.inner.length_of_array_like(context)
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: &mut Context) -> JsResult<bool> {
        self.inner.length_of_array_like(context).map(|len| len == 0)
    }

    /// Push an element to the array.
    pub fn push<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        self.push_items(&[value.into()], context)
    }

    /// Pushes a slice of elements to the array.
    #[inline]
    pub fn push_items(&self, items: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Array::push(&self.inner.clone().into(), items, context)
    }

    /// Pops an element from the array.
    #[inline]
    pub fn pop(&self, context: &mut Context) -> JsResult<JsValue> {
        Array::pop(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.at()`.
    pub fn at<T>(&self, index: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        Array::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    /// Calls `Array.prototype.shift()`.
    #[inline]
    pub fn shift(&self, context: &mut Context) -> JsResult<JsValue> {
        Array::shift(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.unshift()`.
    #[inline]
    pub fn unshift(&self, items: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Array::unshift(&self.inner.clone().into(), items, context)
    }

    /// Calls `Array.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: &mut Context) -> JsResult<Self> {
        Array::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.concat()`.
    #[inline]
    pub fn concat(&self, items: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let object = Array::concat(&self.inner.clone().into(), items, context)?
            .as_object()
            .expect("Array.prototype.filter should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.join()`.
    #[inline]
    pub fn join(&self, separator: Option<JsString>, context: &mut Context) -> JsResult<JsString> {
        Array::join(
            &self.inner.clone().into(),
            &[separator.into_or_undefined()],
            context,
        )
        .map(|x| {
            x.as_string()
                .expect("Array.prototype.join always returns string")
        })
    }

    /// Calls `Array.prototype.fill()`.
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<u32>,
        end: Option<u32>,
        context: &mut Context,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        Array::fill(
            &self.inner.clone().into(),
            &[
                value.into(),
                start.into_or_undefined(),
                end.into_or_undefined(),
            ],
            context,
        )?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: &mut Context,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let index = Array::index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("Array.prototype.indexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: &mut Context,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let index = Array::last_index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("Array.prototype.lastIndexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Array::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = Array::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .expect("Array.prototype.filter should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = Array::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .expect("Array.prototype.map should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.every()`.
    #[inline]
    pub fn every(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = Array::every(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("Array.prototype.every should always return boolean");

        Ok(result)
    }

    /// Calls `Array.prototype.some()`.
    #[inline]
    pub fn some(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = Array::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("Array.prototype.some should always return boolean");

        Ok(result)
    }

    /// Calls `Array.prototype.sort()`.
    #[inline]
    pub fn sort(&self, compare_fn: Option<JsFunction>, context: &mut Context) -> JsResult<Self> {
        Array::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    /// Calls `Array.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<u32>,
        end: Option<u32>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = Array::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?
        .as_object()
        .expect("Array.prototype.slice should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Array::reduce(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.reduceRight()`.
    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Array::reduce_right(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.toReversed`.
    #[inline]
    pub fn to_reversed(&self, context: &mut Context) -> JsResult<Self> {
        let array = Array::to_reversed(&self.inner.clone().into(), &[], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .expect("`to_reversed` must always return an `Array` on success"),
        })
    }

    /// Calls `Array.prototype.toSorted`.
    #[inline]
    pub fn to_sorted(
        &self,
        compare_fn: Option<JsFunction>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let array = Array::to_sorted(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: array
                .as_object()
                .expect("`to_sorted` must always return an `Array` on success"),
        })
    }

    /// Calls `Array.prototype.with`.
    #[inline]
    pub fn with(&self, index: u64, value: JsValue, context: &mut Context) -> JsResult<Self> {
        let array = Array::with(&self.inner.clone().into(), &[index.into(), value], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .expect("`with` must always return an `Array` on success"),
        })
    }
}

impl From<JsArray> for JsObject {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone()
    }
}

impl From<JsArray> for JsValue {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsArray {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not an Array object")
                .into())
        }
    }
}
