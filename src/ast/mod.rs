// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

mod ty;
mod id;
mod serialize;

pub use ty::{Alternative, Alternatives, Fields, KeyTy, Ty, Variant, Variants};

pub mod inner {
    pub use ty::TyInner;

    use super::ty;
}