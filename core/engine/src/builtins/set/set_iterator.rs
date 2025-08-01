//! This module implements the `SetIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects

use super::ordered_set::{OrderedSet, SetLock};
use crate::{
    Context, JsData, JsResult,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject, JsValue, iterable::create_iter_result_object,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

/// The Set Iterator object represents an iteration over a set. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects
#[derive(Debug, Finalize, Trace, JsData)]
pub(crate) struct SetIterator {
    iterated_set: JsValue,
    next_index: usize,
    #[unsafe_ignore_trace]
    iteration_kind: PropertyNameKind,
    lock: SetLock,
}

impl IntrinsicObject for SetIterator {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Set Iterator"),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().set()
    }
}

impl SetIterator {
    /// Constructs a new `SetIterator`, that will iterate over `set`, starting at index 0
    const fn new(set: JsValue, kind: PropertyNameKind, lock: SetLock) -> Self {
        Self {
            iterated_set: set,
            next_index: 0,
            iteration_kind: kind,
            lock,
        }
    }

    /// Abstract operation `CreateSetIterator( set, kind )`
    ///
    /// Creates a new iterator over the given set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createsetiterator
    pub(crate) fn create_set_iterator(
        set: JsValue,
        kind: PropertyNameKind,
        lock: SetLock,
        context: &Context,
    ) -> JsValue {
        let set_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().objects().iterator_prototypes().set(),
            Self::new(set, kind, lock),
        );
        set_iterator.into()
    }

    /// %SetIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%setiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let mut set_iterator = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not an SetIterator"))?;

        // The borrow checker cannot see that we're splitting the `GcRefMut` in two
        // disjointed parts. However, if we manipulate a `&mut` instead, it can
        // deduce that invariant.
        let set_iterator = &mut *set_iterator;
        {
            let m = &set_iterator.iterated_set;
            let mut index = set_iterator.next_index;
            let item_kind = &set_iterator.iteration_kind;

            if set_iterator.iterated_set.is_undefined() {
                return Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ));
            }

            let object = m.as_object();
            let entries = object
                .as_ref()
                .and_then(|o| o.downcast_ref::<OrderedSet>())
                .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Set"))?;

            let num_entries = entries.full_len();
            while index < num_entries {
                let e = entries.get_index(index);
                index += 1;
                set_iterator.next_index = index;
                if let Some(value) = e {
                    match item_kind {
                        PropertyNameKind::Value => {
                            return Ok(create_iter_result_object(value.clone(), false, context));
                        }
                        PropertyNameKind::KeyAndValue => {
                            let result = Array::create_array_from_list(
                                [value.clone(), value.clone()],
                                context,
                            );
                            return Ok(create_iter_result_object(result.into(), false, context));
                        }
                        PropertyNameKind::Key => {
                            panic!("tried to collect only keys of Set")
                        }
                    }
                }
            }
        }

        set_iterator.iterated_set = JsValue::undefined();
        Ok(create_iter_result_object(
            JsValue::undefined(),
            true,
            context,
        ))
    }
}
