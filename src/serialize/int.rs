// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use serde::ser::{Serialize, Serializer};

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

#[repr(transparent)]
pub struct IntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl IntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        IntSerializer { ptr: ptr }
    }
}

impl<'p> Serialize for IntSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = ffi!(PyLong_AsLongLong(self.ptr));
        if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null()) {
            UIntSerializer::new(self.ptr).serialize(serializer)
        } else {
            serializer.serialize_i64(val)
        }
    }
}

#[repr(transparent)]
pub struct UIntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl UIntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        UIntSerializer { ptr: ptr }
    }
}

impl<'p> Serialize for UIntSerializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ffi!(PyErr_Clear());
        let val = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
        if unlikely!(val == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
            err!(SerializeError::Integer64Bits)
        }
        serializer.serialize_u64(val)
    }
}

#[repr(transparent)]
pub struct Int53Serializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl Int53Serializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        Int53Serializer { ptr: ptr }
    }
}

impl<'p> Serialize for Int53Serializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = ffi!(PyLong_AsLongLong(self.ptr));
        if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null())
            || (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
        {
            err!(SerializeError::Integer53Bits)
        }
        serializer.serialize_i64(val)
    }
}
