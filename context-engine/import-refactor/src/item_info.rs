use syn;

// Trait for extracting common information from individual item types
pub trait ItemInfo {
    fn get_visibility(&self) -> &syn::Visibility;
    fn get_attributes(&self) -> &[syn::Attribute];
    fn get_identifier(&self) -> Option<String>;

    fn is_public(&self) -> bool {
        matches!(self.get_visibility(), syn::Visibility::Public(_))
    }
}

impl ItemInfo for syn::ItemFn {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.sig.ident.to_string())
    }
}

impl ItemInfo for syn::ItemStruct {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemEnum {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemType {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemConst {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemStatic {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemMod {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemTrait {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        Some(self.ident.to_string())
    }
}

impl ItemInfo for syn::ItemUse {
    fn get_visibility(&self) -> &syn::Visibility {
        &self.vis
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        // Use statements don't have a single identifier
        None
    }
}

impl ItemInfo for syn::ItemMacro {
    fn get_visibility(&self) -> &syn::Visibility {
        // Macros don't have visibility in the traditional sense
        // Return the inherited visibility since macros with macro_export
        // are handled specially in the is_public check
        &syn::Visibility::Inherited
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    fn get_identifier(&self) -> Option<String> {
        // Only include if it has macro_export
        if has_macro_export_attribute(&self.attrs) {
            self.ident.as_ref().map(|i| i.to_string())
        } else {
            None
        }
    }

    fn is_public(&self) -> bool {
        // Macros are considered public only if they have macro_export
        has_macro_export_attribute(&self.attrs)
    }
}

impl ItemInfo for syn::Item {
    fn get_visibility(&self) -> &syn::Visibility {
        match self {
            syn::Item::Fn(item) => item.get_visibility(),
            syn::Item::Struct(item) => item.get_visibility(),
            syn::Item::Enum(item) => item.get_visibility(),
            syn::Item::Type(item) => item.get_visibility(),
            syn::Item::Const(item) => item.get_visibility(),
            syn::Item::Static(item) => item.get_visibility(),
            syn::Item::Mod(item) => item.get_visibility(),
            syn::Item::Trait(item) => item.get_visibility(),
            syn::Item::Use(item) => item.get_visibility(),
            syn::Item::Macro(item) => item.get_visibility(),
            _ => &syn::Visibility::Inherited,
        }
    }

    fn get_attributes(&self) -> &[syn::Attribute] {
        match self {
            syn::Item::Fn(item) => item.get_attributes(),
            syn::Item::Struct(item) => item.get_attributes(),
            syn::Item::Enum(item) => item.get_attributes(),
            syn::Item::Type(item) => item.get_attributes(),
            syn::Item::Const(item) => item.get_attributes(),
            syn::Item::Static(item) => item.get_attributes(),
            syn::Item::Mod(item) => item.get_attributes(),
            syn::Item::Trait(item) => item.get_attributes(),
            syn::Item::Use(item) => item.get_attributes(),
            syn::Item::Macro(item) => item.get_attributes(),
            _ => &[],
        }
    }

    fn get_identifier(&self) -> Option<String> {
        match self {
            syn::Item::Fn(item) => item.get_identifier(),
            syn::Item::Struct(item) => item.get_identifier(),
            syn::Item::Enum(item) => item.get_identifier(),
            syn::Item::Type(item) => item.get_identifier(),
            syn::Item::Const(item) => item.get_identifier(),
            syn::Item::Static(item) => item.get_identifier(),
            syn::Item::Mod(item) => item.get_identifier(),
            syn::Item::Trait(item) => item.get_identifier(),
            syn::Item::Use(item) => item.get_identifier(),
            syn::Item::Macro(item) => item.get_identifier(),
            _ => None,
        }
    }
}

// Helper function to check for macro_export attribute
pub fn has_macro_export_attribute(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("macro_export") {
            return true;
        }
    }
    false
}
