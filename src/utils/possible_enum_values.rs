use itertools::Itertools;
use std::{fmt, marker::PhantomData};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

/// A helper to retrieve all possible variants/values of an enum. Supports
/// providing custom names for specific variants and hiding specific variants.
pub struct PossibleEnumValues<E, N = NoCustomVariantNames, F = NoHidden> {
    enum_type: PhantomData<E>,
    support_custom_variant_names: PhantomData<N>,
    support_hiding_variants: PhantomData<F>,
}

// Type-States

#[derive(Clone)]
pub struct NoCustomVariantNames;
#[derive(Clone)]
pub struct CustomVariantNames;

#[derive(Clone)]
pub struct NoHidden;
#[derive(Clone)]
pub struct Hidden;

// Builder pattern

impl<E> PossibleEnumValues<E> {
    /// Creates a new instance of the `PossibleEnumValues` builder.
    pub fn new() -> PossibleEnumValues<E, NoCustomVariantNames, NoHidden> {
        PossibleEnumValues {
            enum_type: PhantomData,
            support_custom_variant_names: PhantomData,
            support_hiding_variants: PhantomData,
        }
    }
}

impl<E, N, H> PossibleEnumValues<E, N, H> {
    /// Support overriding default variant names with custom names with:
    /// `#[strum(message = "<custom-name>")]`.
    pub fn custom_names(self) -> PossibleEnumValues<E, CustomVariantNames, H> {
        PossibleEnumValues {
            enum_type: self.enum_type,
            support_custom_variant_names: PhantomData,
            support_hiding_variants: self.support_hiding_variants,
        }
    }

    /// Support hiding variants with:
    /// `#[strum(props(Hidden = "true"))]`.
    pub fn hidden(self) -> PossibleEnumValues<E, N, Hidden> {
        PossibleEnumValues {
            enum_type: self.enum_type,
            support_custom_variant_names: self.support_custom_variant_names,
            support_hiding_variants: PhantomData,
        }
    }
}

// Hide configured variants

impl<E, N> PossibleEnumValues<E, N, Hidden> {
    /// Returns `true` if a variant should be shown, and `false` if it should
    /// be hidden.
    fn should_be_shown(variant: &E) -> bool
    where
        E: EnumProperty,
    {
        // TODO: replace with strum's get_bool once available
        !matches!(variant.get_str("Hidden"), Some("true"))
    }
}

// Get variant as string

impl<E, H> PossibleEnumValues<E, NoCustomVariantNames, H> {
    /// Get the string representation of an enum variant. Use `to_string`.
    fn get_variant_as_string(variant: E) -> String
    where
        E: fmt::Display,
    {
        variant.to_string()
    }
}

impl<E, H> PossibleEnumValues<E, CustomVariantNames, H> {
    /// Get the string representation of an enum variant. Use strum's `message`
    /// if available, otherwise use `to_string`.
    fn get_variant_as_string(variant: E) -> String
    where
        E: EnumMessage + fmt::Display,
    {
        variant
            .get_message()
            .map(str::to_owned)
            .unwrap_or_else(|| variant.to_string())
    }
}

// Get final string list of variants

impl<E> PossibleEnumValues<E, NoCustomVariantNames, NoHidden> {
    /// Get a string list of all possible variants for the enum.
    pub fn get(self) -> String
    where
        E: IntoEnumIterator + fmt::Display,
    {
        E::iter().map(Self::get_variant_as_string).join(", ")
    }
}

impl<E> PossibleEnumValues<E, CustomVariantNames, NoHidden> {
    /// Get a string list of all possible variants for the enum.
    pub fn get(self) -> String
    where
        E: IntoEnumIterator + EnumMessage + fmt::Display,
    {
        E::iter().map(Self::get_variant_as_string).join(", ")
    }
}

impl<E> PossibleEnumValues<E, NoCustomVariantNames, Hidden> {
    /// Get a string list of all possible variants for the enum.
    pub fn get(self) -> String
    where
        E: IntoEnumIterator + EnumProperty + fmt::Display,
    {
        E::iter()
            .filter(Self::should_be_shown)
            .map(Self::get_variant_as_string)
            .join(", ")
    }
}

impl<E> PossibleEnumValues<E, CustomVariantNames, Hidden> {
    /// Get a string list of all possible variants for the enum.
    pub fn _get(self) -> String
    where
        E: IntoEnumIterator + EnumMessage + EnumProperty + fmt::Display,
    {
        E::iter()
            .filter(Self::should_be_shown)
            .map(Self::get_variant_as_string)
            .join(", ")
    }
}
