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

//! Gravel is a data type library which may reference other libraries.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::ops::Deref;

use amplify::confinement::{Confined, TinyOrdMap};
use amplify::Wrapper;

use crate::ast::{NestedRef, TranslateError};
use crate::{Ident, SemVer, StenType, Translate, Ty, TyId, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum GravelTy {
    #[from]
    Name(TypeName),

    #[from]
    Inline(Box<Ty<GravelTy>>),

    Extern(TypeName, GravelAlias),
}

impl TypeRef for GravelTy {}

impl Deref for GravelTy {
    type Target = Ty<Self>;

    fn deref(&self) -> &Self::Target { self.as_ty() }
}

impl NestedRef for GravelTy {
    fn as_ty(&self) -> &Ty<Self> {
        match self {
            GravelTy::Name(_) => &Ty::UNIT,
            GravelTy::Inline(ty) => ty.as_ref(),
            GravelTy::Extern(_, _) => &Ty::UNIT,
        }
    }

    fn into_ty(self) -> Ty<Self> {
        match self {
            GravelTy::Name(_) => Ty::UNIT,
            GravelTy::Inline(ty) => *ty,
            GravelTy::Extern(_, _) => Ty::UNIT,
        }
    }

    fn about(&self) -> String { todo!() }
}

impl Display for GravelTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GravelTy::Name(name) => write!(f, "{}", name),
            GravelTy::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            GravelTy::Inline(ty) => write!(f, "{}", ty),
            GravelTy::Extern(name, lib) => write!(f, "{}.{}", lib, name),
        }
    }
}

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct GravelId(blake3::Hash);

impl Ord for GravelId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for GravelId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for GravelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.as_bytes()[14..18]);
            write!(f, "{}#{}", self.0, m)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

pub struct Hasher(blake3::Hasher);

// TODO: Use real tag
pub const GRAVEL_ID_TAG: [u8; 32] = [0u8; 32];

impl Hasher {
    pub fn new() -> Hasher { Hasher(blake3::Hasher::new_keyed(&GRAVEL_ID_TAG)) }

    pub fn input(&mut self, id: TyId) {
        self.0.write_all(id.as_bytes()).expect("hashers do not error")
    }

    pub fn finish(self) -> GravelId { GravelId(self.0.finalize()) }
}

pub type GravelAlias = Ident;
pub type GravelName = Ident;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display("typelib {name}@{ver} {id:#}")]
pub struct Dependency {
    pub id: GravelId,
    pub name: Ident,
    pub ver: SemVer,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Gravel {
    pub roots: BTreeSet<TyId>,
    pub dependencies: TinyOrdMap<GravelAlias, Dependency>,
    pub types: Confined<BTreeMap<TypeName, Ty<GravelTy>>, 1, { u16::MAX as usize }>,
}

impl TryFrom<StenType> for Gravel {
    type Error = TranslateError;

    fn try_from(root: StenType) -> Result<Self, Self::Error> { root.translate(&mut ()) }
}

impl Display for Gravel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.types {
            writeln!(f, "data {:16} :: {}", name, ty)?;
        }
        Ok(())
    }
}

impl Gravel {
    pub fn id(&self) -> GravelId {
        let mut hasher = Hasher::new();
        for id in self.roots.iter() {
            hasher.input(*id);
        }
        hasher.finish()
    }
}