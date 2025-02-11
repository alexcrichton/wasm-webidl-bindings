use crate::text;
use id_arena::{Arena, Id};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WebidlBindings {
    pub types: WebidlTypes,
    pub bindings: FunctionBindings,
    pub binds: Binds,
}

impl walrus::CustomSection for WebidlBindings {
    fn name(&self) -> &str {
        "webidl-bindings"
    }

    fn data(&self, ids_to_indices: &walrus::IdsToIndices) -> Cow<[u8]> {
        let mut data = vec![];
        crate::binary::encode(self, ids_to_indices, &mut data)
            .expect("writing into a vec never fails");
        data.into()
    }
}

macro_rules! id_newtypes {
    ( $( $name:ident($inner:ty), )* ) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            pub struct $name(pub(crate) Id<$inner>);

            impl From<$name> for Id<$inner> {
                #[inline]
                fn from(id: $name) -> Id<$inner> {
                    id.0
                }
            }
        )*
    }
}

id_newtypes! {
    WebidlFunctionId(WebidlCompoundType),
    WebidlDictionaryId(WebidlCompoundType),
    WebidlEnumerationId(WebidlCompoundType),
    WebidlUnionId(WebidlCompoundType),

    ImportBindingId(FunctionBinding),
    ExportBindingId(FunctionBinding),
}

#[derive(Debug, Default)]
pub struct WebidlTypes {
    pub(crate) names: HashMap<String, Id<WebidlCompoundType>>,
    indices: Vec<Id<WebidlCompoundType>>,
    pub(crate) arena: Arena<WebidlCompoundType>,
}

pub trait WebidlTypeId: Into<WebidlCompoundType> {
    type Id: Into<Id<WebidlCompoundType>>;

    #[doc(hidden)]
    fn wrap(id: Id<WebidlCompoundType>) -> Self::Id;
    #[doc(hidden)]
    fn get(ty: &WebidlCompoundType) -> Option<&Self>;
    #[doc(hidden)]
    fn get_mut(ty: &mut WebidlCompoundType) -> Option<&mut Self>;
}

macro_rules! impl_webidl_type_id {
    ( $( $id:ident => $variant:ident($ty:ty); )* ) => {
        $(
            impl WebidlTypeId for $ty {
                type Id = $id;

                fn wrap(id: Id<WebidlCompoundType>) -> Self::Id {
                    $id(id)
                }

                fn get(ty: &WebidlCompoundType) -> Option<&Self> {
                    if let WebidlCompoundType::$variant(x) = ty {
                        Some(x)
                    } else {
                        None
                    }
                }

                fn get_mut(ty: &mut WebidlCompoundType) -> Option<&mut Self> {
                    if let WebidlCompoundType::$variant(x) = ty {
                        Some(x)
                    } else {
                        None
                    }
                }
            }

            impl From<$id> for WebidlTypeRef {
                fn from(x: $id) -> WebidlTypeRef {
                    WebidlTypeRef::Id(x.into())
                }
            }
        )*
    }
}

impl_webidl_type_id! {
    WebidlFunctionId => Function(WebidlFunction);
    WebidlDictionaryId => Dictionary(WebidlDictionary);
    WebidlEnumerationId => Enumeration(WebidlEnumeration);
    WebidlUnionId => Union(WebidlUnion);
}

impl WebidlTypes {
    pub fn by_name(&self, name: &str) -> Option<Id<WebidlCompoundType>> {
        self.names.get(name).cloned()
    }

    pub fn by_index(&self, index: u32) -> Option<Id<WebidlCompoundType>> {
        self.indices.get(index as usize).cloned()
    }

    pub fn get<T>(&self, id: T::Id) -> Option<&T>
    where
        T: WebidlTypeId,
    {
        self.arena.get(id.into()).and_then(T::get)
    }

    pub fn get_mut<T>(&mut self, id: T::Id) -> Option<&mut T>
    where
        T: WebidlTypeId,
    {
        self.arena.get_mut(id.into()).and_then(T::get_mut)
    }

    pub fn insert<T>(&mut self, ty: T) -> T::Id
    where
        T: WebidlTypeId,
    {
        let id = self.arena.alloc(ty.into());
        self.indices.push(id);
        T::wrap(id)
    }
}

#[derive(Debug, Default)]
pub struct FunctionBindings {
    pub(crate) names: HashMap<String, Id<FunctionBinding>>,
    indices: Vec<Id<FunctionBinding>>,
    pub(crate) arena: Arena<FunctionBinding>,
}

pub trait FunctionBindingId: Into<FunctionBinding> {
    type Id: Into<Id<FunctionBinding>>;

    #[doc(hidden)]
    fn wrap(id: Id<FunctionBinding>) -> Self::Id;
    #[doc(hidden)]
    fn get(b: &FunctionBinding) -> Option<&Self>;
    #[doc(hidden)]
    fn get_mut(b: &mut FunctionBinding) -> Option<&mut Self>;
}

macro_rules! impl_function_binding_id {
    ( $( $id:ident => $variant:ident($ty:ident); )* ) => {
        $(
            impl FunctionBindingId for $ty {
                type Id = $id;

                fn wrap(id: Id<FunctionBinding>) -> Self::Id {
                    $id(id)
                }

                fn get(ty: &FunctionBinding) -> Option<&Self> {
                    if let FunctionBinding::$variant(x) = ty {
                        Some(x)
                    } else {
                        None
                    }
                }

                fn get_mut(ty: &mut FunctionBinding) -> Option<&mut Self> {
                    if let FunctionBinding::$variant(x) = ty {
                        Some(x)
                    } else {
                        None
                    }
                }
            }
        )*
    }
}

impl_function_binding_id! {
    ImportBindingId => Import(ImportBinding);
    ExportBindingId => Export(ExportBinding);
}

impl FunctionBindings {
    pub fn by_name(&self, name: &str) -> Option<Id<FunctionBinding>> {
        self.names.get(name).cloned()
    }

    pub fn by_index(&self, index: u32) -> Option<Id<FunctionBinding>> {
        self.indices.get(index as usize).cloned()
    }

    pub fn get<T>(&self, id: T::Id) -> Option<&T>
    where
        T: FunctionBindingId,
    {
        self.arena.get(id.into()).and_then(T::get)
    }

    pub fn get_mut<T>(&mut self, id: T::Id) -> Option<&mut T>
    where
        T: FunctionBindingId,
    {
        self.arena.get_mut(id.into()).and_then(T::get_mut)
    }

    pub fn insert<T>(&mut self, binding: T) -> T::Id
    where
        T: FunctionBindingId,
    {
        let id = self.arena.alloc(binding.into());
        self.indices.push(id);
        T::wrap(id)
    }
}

#[derive(Debug, Default)]
pub struct Binds {
    pub(crate) arena: id_arena::Arena<Bind>,
}

impl Binds {
    pub fn get(&self, id: Id<Bind>) -> Option<&Bind> {
        self.arena.get(id.into())
    }

    pub fn get_mut(&mut self, id: Id<Bind>) -> Option<&mut Bind> {
        self.arena.get_mut(id.into())
    }

    pub fn insert(&mut self, bind: Bind) -> Id<Bind> {
        self.arena.alloc(bind)
    }
}

#[derive(Debug)]
pub struct BuildAstActions<'a> {
    section: &'a mut WebidlBindings,
    module: &'a walrus::Module,
    ids: &'a walrus::IndicesToIds,
}

impl<'a> BuildAstActions<'a> {
    pub fn new(
        section: &'a mut WebidlBindings,
        module: &'a walrus::Module,
        ids: &'a walrus::IndicesToIds,
    ) -> Self {
        BuildAstActions {
            section,
            module,
            ids,
        }
    }
}

impl<'a> text::Actions for BuildAstActions<'a> {
    type WebidlBindingsSection = ();
    fn webidl_bindings_section(&mut self, _types: (), _bindings: ()) {}

    type WebidlTypeSubsection = ();
    fn webidl_type_subsection(&mut self, _types: Vec<()>) {}

    type WebidlType = ();
    fn webidl_type(&mut self, name: Option<&str>, ty: Id<WebidlCompoundType>) {
        if let Some(name) = name {
            self.section.types.names.insert(name.to_string(), ty);
        }
    }

    type WebidlCompoundType = Id<WebidlCompoundType>;

    type WebidlFunction = WebidlFunctionId;
    fn webidl_function(
        &mut self,
        kind: Option<WebidlFunctionKind>,
        params: Option<Vec<WebidlTypeRef>>,
        result: Option<WebidlTypeRef>,
    ) -> WebidlFunctionId {
        let kind = kind.unwrap_or(WebidlFunctionKind::Static);
        let params = params.unwrap_or(vec![]);
        self.section.types.insert(WebidlFunction {
            kind,
            params,
            result,
        })
    }

    type WebidlFunctionKind = WebidlFunctionKind;

    type WebidlFunctionKindMethod = WebidlFunctionKindMethod;
    fn webidl_function_kind_method(&mut self, ty: WebidlTypeRef) -> WebidlFunctionKindMethod {
        WebidlFunctionKindMethod { ty }
    }

    type WebidlFunctionKindConstructor = WebidlFunctionKind;
    fn webidl_function_kind_constructor_default_new_target(&mut self) -> WebidlFunctionKind {
        WebidlFunctionKind::Constructor
    }

    type WebidlFunctionParams = Vec<WebidlTypeRef>;
    fn webidl_function_params(&mut self, tys: Vec<WebidlTypeRef>) -> Vec<WebidlTypeRef> {
        tys
    }

    type WebidlFunctionResult = WebidlTypeRef;
    fn webidl_function_result(&mut self, ty: WebidlTypeRef) -> WebidlTypeRef {
        ty
    }

    type WebidlDictionary = WebidlDictionaryId;
    fn webidl_dictionary(&mut self, fields: Vec<WebidlDictionaryField>) -> WebidlDictionaryId {
        self.section.types.insert(WebidlDictionary { fields })
    }

    type WebidlDictionaryField = WebidlDictionaryField;
    fn webidl_dictionary_field(
        &mut self,
        name: String,
        ty: WebidlTypeRef,
    ) -> WebidlDictionaryField {
        WebidlDictionaryField { name, ty }
    }

    type WebidlDictionaryFieldName = String;
    fn webidl_dictionary_field_name(&mut self, name: &str) -> String {
        name.into()
    }

    type WebidlEnumeration = WebidlEnumerationId;
    fn webidl_enumeration(&mut self, values: Vec<String>) -> WebidlEnumerationId {
        self.section.types.insert(WebidlEnumeration { values })
    }

    type WebidlEnumerationValue = String;
    fn webidl_enumeration_value(&mut self, value: &str) -> String {
        value.into()
    }

    type WebidlUnion = WebidlUnionId;
    fn webidl_union(&mut self, members: Vec<WebidlTypeRef>) -> WebidlUnionId {
        self.section.types.insert(WebidlUnion { members })
    }

    type WebidlFunctionBindingsSubsection = ();
    fn webidl_function_bindings_subsection(&mut self, _bindings: Vec<()>, _binds: Vec<()>) {}

    type FunctionBinding = ();

    type ImportBinding = ();
    fn import_binding(
        &mut self,
        name: Option<&str>,
        wasm_ty: walrus::TypeId,
        webidl_ty: WebidlTypeRef,
        params: OutgoingBindingMap,
        result: IncomingBindingMap,
    ) {
        let id: ImportBindingId = self.section.bindings.insert(ImportBinding {
            wasm_ty,
            webidl_ty,
            params,
            result,
        });
        if let Some(name) = name {
            self.section
                .bindings
                .names
                .insert(name.to_string(), id.into());
        }
    }

    type ExportBinding = ();
    fn export_binding(
        &mut self,
        name: Option<&str>,
        wasm_ty: walrus::TypeId,
        webidl_ty: WebidlTypeRef,
        params: IncomingBindingMap,
        result: OutgoingBindingMap,
    ) {
        let id: ExportBindingId = self.section.bindings.insert(ExportBinding {
            wasm_ty,
            webidl_ty,
            params,
            result,
        });
        if let Some(name) = name {
            self.section
                .bindings
                .names
                .insert(name.to_string(), id.into());
        }
    }

    type Bind = ();
    fn bind(&mut self, func: walrus::FunctionId, binding: Id<FunctionBinding>) {
        self.section.binds.insert(Bind { func, binding });
    }

    type OutgoingBindingMap = OutgoingBindingMap;
    fn outgoing_binding_map(
        &mut self,
        bindings: Vec<OutgoingBindingExpression>,
    ) -> OutgoingBindingMap {
        OutgoingBindingMap { bindings }
    }

    type IncomingBindingMap = IncomingBindingMap;
    fn incoming_binding_map(
        &mut self,
        bindings: Vec<IncomingBindingExpression>,
    ) -> IncomingBindingMap {
        IncomingBindingMap { bindings }
    }

    type OutgoingBindingExpression = OutgoingBindingExpression;

    type OutgoingBindingExpressionAs = OutgoingBindingExpressionAs;
    fn outgoing_binding_expression_as(
        &mut self,
        ty: WebidlTypeRef,
        idx: u32,
    ) -> OutgoingBindingExpressionAs {
        OutgoingBindingExpressionAs { ty, idx }
    }

    type OutgoingBindingExpressionUtf8Str = OutgoingBindingExpressionUtf8Str;
    fn outgoing_binding_expression_utf8_str(
        &mut self,
        ty: WebidlTypeRef,
        offset: u32,
        length: u32,
    ) -> OutgoingBindingExpressionUtf8Str {
        OutgoingBindingExpressionUtf8Str { ty, offset, length }
    }

    type OutgoingBindingExpressionUtf8CStr = OutgoingBindingExpressionUtf8CStr;
    fn outgoing_binding_expression_utf8_c_str(
        &mut self,
        ty: WebidlTypeRef,
        offset: u32,
    ) -> OutgoingBindingExpressionUtf8CStr {
        OutgoingBindingExpressionUtf8CStr { ty, offset }
    }

    type OutgoingBindingExpressionI32ToEnum = OutgoingBindingExpressionI32ToEnum;
    fn outgoing_binding_expression_i32_to_enum(
        &mut self,
        ty: WebidlTypeRef,
        idx: u32,
    ) -> OutgoingBindingExpressionI32ToEnum {
        OutgoingBindingExpressionI32ToEnum { ty, idx }
    }

    type OutgoingBindingExpressionView = OutgoingBindingExpressionView;
    fn outgoing_binding_expression_view(
        &mut self,
        ty: WebidlTypeRef,
        offset: u32,
        length: u32,
    ) -> OutgoingBindingExpressionView {
        OutgoingBindingExpressionView { ty, offset, length }
    }

    type OutgoingBindingExpressionCopy = OutgoingBindingExpressionCopy;
    fn outgoing_binding_expression_copy(
        &mut self,
        ty: WebidlTypeRef,
        offset: u32,
        length: u32,
    ) -> OutgoingBindingExpressionCopy {
        OutgoingBindingExpressionCopy { ty, offset, length }
    }

    type OutgoingBindingExpressionDict = OutgoingBindingExpressionDict;
    fn outgoing_binding_expression_dict(
        &mut self,
        ty: WebidlTypeRef,
        fields: Vec<OutgoingBindingExpression>,
    ) -> OutgoingBindingExpressionDict {
        OutgoingBindingExpressionDict { ty, fields }
    }

    type OutgoingBindingExpressionBindExport = OutgoingBindingExpressionBindExport;
    fn outgoing_binding_expression_bind_export(
        &mut self,
        ty: WebidlTypeRef,
        binding: Id<FunctionBinding>,
        idx: u32,
    ) -> OutgoingBindingExpressionBindExport {
        OutgoingBindingExpressionBindExport { ty, binding, idx }
    }

    type IncomingBindingExpression = IncomingBindingExpression;

    type IncomingBindingExpressionGet = IncomingBindingExpressionGet;
    fn incoming_binding_expression_get(&mut self, idx: u32) -> IncomingBindingExpressionGet {
        IncomingBindingExpressionGet { idx }
    }

    type IncomingBindingExpressionAs = IncomingBindingExpressionAs;
    fn incoming_binding_expression_as(
        &mut self,
        ty: walrus::ValType,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionAs {
        let expr = Box::new(expr);
        IncomingBindingExpressionAs { ty, expr }
    }

    type IncomingBindingExpressionAllocUtf8Str = IncomingBindingExpressionAllocUtf8Str;
    fn incoming_binding_expression_alloc_utf8_str(
        &mut self,
        alloc_func_name: &str,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionAllocUtf8Str {
        let alloc_func_name = alloc_func_name.into();
        let expr = Box::new(expr);
        IncomingBindingExpressionAllocUtf8Str {
            alloc_func_name,
            expr,
        }
    }

    type IncomingBindingExpressionAllocCopy = IncomingBindingExpressionAllocCopy;
    fn incoming_binding_expression_alloc_copy(
        &mut self,
        alloc_func_name: &str,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionAllocCopy {
        let alloc_func_name = alloc_func_name.into();
        let expr = Box::new(expr);
        IncomingBindingExpressionAllocCopy {
            alloc_func_name,
            expr,
        }
    }

    type IncomingBindingExpressionEnumToI32 = IncomingBindingExpressionEnumToI32;
    fn incoming_binding_expression_enum_to_i32(
        &mut self,
        ty: WebidlTypeRef,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionEnumToI32 {
        let expr = Box::new(expr);
        IncomingBindingExpressionEnumToI32 { ty, expr }
    }

    type IncomingBindingExpressionField = IncomingBindingExpressionField;
    fn incoming_binding_expression_field(
        &mut self,
        idx: u32,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionField {
        let expr = Box::new(expr);
        IncomingBindingExpressionField { idx, expr }
    }

    type IncomingBindingExpressionBindImport = IncomingBindingExpressionBindImport;
    fn incoming_binding_expression_bind_import(
        &mut self,
        ty: walrus::TypeId,
        binding: Id<FunctionBinding>,
        expr: IncomingBindingExpression,
    ) -> IncomingBindingExpressionBindImport {
        let expr = Box::new(expr);
        IncomingBindingExpressionBindImport { ty, binding, expr }
    }

    type WebidlTypeRef = WebidlTypeRef;

    type WebidlTypeRefNamed = WebidlTypeRef;
    fn webidl_type_ref_named(&mut self, name: &str) -> Option<WebidlTypeRef> {
        self.section.types.by_name(name).map(Into::into)
    }

    type WebidlTypeRefIndexed = WebidlTypeRef;
    fn webidl_type_ref_indexed(&mut self, idx: u32) -> Option<WebidlTypeRef> {
        self.section.types.by_index(idx).map(Into::into)
    }

    type WebidlScalarType = WebidlScalarType;
    fn webidl_scalar_type_any(&mut self) -> WebidlScalarType {
        WebidlScalarType::Any
    }
    fn webidl_scalar_type_boolean(&mut self) -> WebidlScalarType {
        WebidlScalarType::Boolean
    }
    fn webidl_scalar_type_byte(&mut self) -> WebidlScalarType {
        WebidlScalarType::Byte
    }
    fn webidl_scalar_type_octet(&mut self) -> WebidlScalarType {
        WebidlScalarType::Octet
    }
    fn webidl_scalar_type_long(&mut self) -> WebidlScalarType {
        WebidlScalarType::Long
    }
    fn webidl_scalar_type_unsigned_long(&mut self) -> WebidlScalarType {
        WebidlScalarType::UnsignedLong
    }
    fn webidl_scalar_type_short(&mut self) -> WebidlScalarType {
        WebidlScalarType::Short
    }
    fn webidl_scalar_type_unsigned_short(&mut self) -> WebidlScalarType {
        WebidlScalarType::UnsignedShort
    }
    fn webidl_scalar_type_long_long(&mut self) -> WebidlScalarType {
        WebidlScalarType::LongLong
    }
    fn webidl_scalar_type_unsigned_long_long(&mut self) -> WebidlScalarType {
        WebidlScalarType::UnsignedLongLong
    }
    fn webidl_scalar_type_float(&mut self) -> WebidlScalarType {
        WebidlScalarType::Float
    }
    fn webidl_scalar_type_unrestricted_float(&mut self) -> WebidlScalarType {
        WebidlScalarType::UnrestrictedFloat
    }
    fn webidl_scalar_type_double(&mut self) -> WebidlScalarType {
        WebidlScalarType::Double
    }
    fn webidl_scalar_type_unrestricted_double(&mut self) -> WebidlScalarType {
        WebidlScalarType::UnrestrictedDouble
    }
    fn webidl_scalar_type_dom_string(&mut self) -> WebidlScalarType {
        WebidlScalarType::DomString
    }
    fn webidl_scalar_type_byte_string(&mut self) -> WebidlScalarType {
        WebidlScalarType::ByteString
    }
    fn webidl_scalar_type_usv_string(&mut self) -> WebidlScalarType {
        WebidlScalarType::UsvString
    }
    fn webidl_scalar_type_object(&mut self) -> WebidlScalarType {
        WebidlScalarType::Object
    }
    fn webidl_scalar_type_symbol(&mut self) -> WebidlScalarType {
        WebidlScalarType::Symbol
    }
    fn webidl_scalar_type_array_buffer(&mut self) -> WebidlScalarType {
        WebidlScalarType::ArrayBuffer
    }
    fn webidl_scalar_type_data_view(&mut self) -> WebidlScalarType {
        WebidlScalarType::DataView
    }
    fn webidl_scalar_type_int8_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Int8Array
    }
    fn webidl_scalar_type_int16_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Int16Array
    }
    fn webidl_scalar_type_int32_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Int32Array
    }
    fn webidl_scalar_type_uint8_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Uint8Array
    }
    fn webidl_scalar_type_uint16_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Uint16Array
    }
    fn webidl_scalar_type_uint32_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Uint32Array
    }
    fn webidl_scalar_type_uint8_clamped_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Uint8ClampedArray
    }
    fn webidl_scalar_type_float32_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Float32Array
    }
    fn webidl_scalar_type_float64_array(&mut self) -> WebidlScalarType {
        WebidlScalarType::Float64Array
    }

    type WasmValType = walrus::ValType;
    fn wasm_val_type_i32(&mut self) -> walrus::ValType {
        walrus::ValType::I32
    }
    fn wasm_val_type_i64(&mut self) -> walrus::ValType {
        walrus::ValType::I64
    }
    fn wasm_val_type_f32(&mut self) -> walrus::ValType {
        walrus::ValType::F32
    }
    fn wasm_val_type_f64(&mut self) -> walrus::ValType {
        walrus::ValType::F64
    }
    fn wasm_val_type_v128(&mut self) -> walrus::ValType {
        walrus::ValType::V128
    }
    fn wasm_val_type_anyref(&mut self) -> walrus::ValType {
        walrus::ValType::Anyref
    }

    type WasmFuncTypeRef = walrus::TypeId;

    type WasmFuncTypeRefNamed = walrus::TypeId;
    fn wasm_func_type_ref_named(&mut self, name: &str) -> Option<walrus::TypeId> {
        self.module.types.by_name(name)
    }

    type WasmFuncTypeRefIndexed = walrus::TypeId;
    fn wasm_func_type_ref_indexed(&mut self, idx: u32) -> Option<walrus::TypeId> {
        self.ids.get_type(idx).ok()
    }

    type WasmFuncRef = walrus::FunctionId;

    type WasmFuncRefNamed = walrus::FunctionId;
    fn wasm_func_ref_named(&mut self, name: &str) -> Option<walrus::FunctionId> {
        self.module.funcs.by_name(name)
    }

    type WasmFuncRefIndexed = walrus::FunctionId;
    fn wasm_func_ref_indexed(&mut self, idx: u32) -> Option<walrus::FunctionId> {
        self.ids.get_func(idx).ok()
    }

    type BindingRef = Id<FunctionBinding>;

    type BindingRefNamed = Id<FunctionBinding>;
    fn binding_ref_named(&mut self, name: &str) -> Option<Id<FunctionBinding>> {
        self.section.bindings.by_name(name)
    }

    type BindingRefIndexed = Id<FunctionBinding>;
    fn binding_ref_indexed(&mut self, idx: u32) -> Option<Id<FunctionBinding>> {
        self.section.bindings.by_index(idx)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlType {
    pub name: Option<String>,
    pub ty: WebidlCompoundType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebidlCompoundType {
    Function(WebidlFunction),
    Dictionary(WebidlDictionary),
    Enumeration(WebidlEnumeration),
    Union(WebidlUnion),
}

impl From<WebidlFunction> for WebidlCompoundType {
    fn from(a: WebidlFunction) -> Self {
        WebidlCompoundType::Function(a)
    }
}

impl From<WebidlDictionary> for WebidlCompoundType {
    fn from(a: WebidlDictionary) -> Self {
        WebidlCompoundType::Dictionary(a)
    }
}

impl From<WebidlEnumeration> for WebidlCompoundType {
    fn from(a: WebidlEnumeration) -> Self {
        WebidlCompoundType::Enumeration(a)
    }
}

impl From<WebidlUnion> for WebidlCompoundType {
    fn from(a: WebidlUnion) -> Self {
        WebidlCompoundType::Union(a)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlFunction {
    pub kind: WebidlFunctionKind,
    pub params: Vec<WebidlTypeRef>,
    pub result: Option<WebidlTypeRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebidlFunctionKind {
    Static,
    Method(WebidlFunctionKindMethod),
    Constructor,
}

impl From<WebidlFunctionKindMethod> for WebidlFunctionKind {
    fn from(a: WebidlFunctionKindMethod) -> Self {
        WebidlFunctionKind::Method(a)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlFunctionKindMethod {
    pub ty: WebidlTypeRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlDictionary {
    pub fields: Vec<WebidlDictionaryField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlDictionaryField {
    pub name: String,
    pub ty: WebidlTypeRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlEnumeration {
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebidlUnion {
    pub members: Vec<WebidlTypeRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionBinding {
    Import(ImportBinding),
    Export(ExportBinding),
}

impl From<ImportBinding> for FunctionBinding {
    fn from(a: ImportBinding) -> Self {
        FunctionBinding::Import(a)
    }
}

impl From<ExportBinding> for FunctionBinding {
    fn from(a: ExportBinding) -> Self {
        FunctionBinding::Export(a)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportBinding {
    pub wasm_ty: walrus::TypeId,
    pub webidl_ty: WebidlTypeRef,
    pub params: OutgoingBindingMap,
    pub result: IncomingBindingMap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportBinding {
    pub wasm_ty: walrus::TypeId,
    pub webidl_ty: WebidlTypeRef,
    pub params: IncomingBindingMap,
    pub result: OutgoingBindingMap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bind {
    pub func: walrus::FunctionId,
    pub binding: Id<FunctionBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingMap {
    pub bindings: Vec<OutgoingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingMap {
    pub bindings: Vec<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutgoingBindingExpression {
    As(OutgoingBindingExpressionAs),
    Utf8Str(OutgoingBindingExpressionUtf8Str),
    Utf8CStr(OutgoingBindingExpressionUtf8CStr),
    I32ToEnum(OutgoingBindingExpressionI32ToEnum),
    View(OutgoingBindingExpressionView),
    Copy(OutgoingBindingExpressionCopy),
    Dict(OutgoingBindingExpressionDict),
    BindExport(OutgoingBindingExpressionBindExport),
}

impl From<OutgoingBindingExpressionAs> for OutgoingBindingExpression {
    fn from(a: OutgoingBindingExpressionAs) -> Self {
        OutgoingBindingExpression::As(a)
    }
}

impl From<OutgoingBindingExpressionUtf8Str> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionUtf8Str) -> Self {
        OutgoingBindingExpression::Utf8Str(s)
    }
}

impl From<OutgoingBindingExpressionUtf8CStr> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionUtf8CStr) -> Self {
        OutgoingBindingExpression::Utf8CStr(s)
    }
}

impl From<OutgoingBindingExpressionI32ToEnum> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionI32ToEnum) -> Self {
        OutgoingBindingExpression::I32ToEnum(s)
    }
}

impl From<OutgoingBindingExpressionView> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionView) -> Self {
        OutgoingBindingExpression::View(s)
    }
}

impl From<OutgoingBindingExpressionCopy> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionCopy) -> Self {
        OutgoingBindingExpression::Copy(s)
    }
}

impl From<OutgoingBindingExpressionDict> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionDict) -> Self {
        OutgoingBindingExpression::Dict(s)
    }
}

impl From<OutgoingBindingExpressionBindExport> for OutgoingBindingExpression {
    fn from(s: OutgoingBindingExpressionBindExport) -> Self {
        OutgoingBindingExpression::BindExport(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionAs {
    pub ty: WebidlTypeRef,
    pub idx: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionUtf8Str {
    pub ty: WebidlTypeRef,
    pub offset: u32,
    pub length: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionUtf8CStr {
    pub ty: WebidlTypeRef,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionI32ToEnum {
    pub ty: WebidlTypeRef,
    pub idx: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionView {
    pub ty: WebidlTypeRef,
    pub offset: u32,
    pub length: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionCopy {
    pub ty: WebidlTypeRef,
    pub offset: u32,
    pub length: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionDict {
    pub ty: WebidlTypeRef,
    pub fields: Vec<OutgoingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingBindingExpressionBindExport {
    pub ty: WebidlTypeRef,
    pub binding: Id<FunctionBinding>,
    pub idx: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncomingBindingExpression {
    Get(IncomingBindingExpressionGet),
    As(IncomingBindingExpressionAs),
    AllocUtf8Str(IncomingBindingExpressionAllocUtf8Str),
    AllocCopy(IncomingBindingExpressionAllocCopy),
    EnumToI32(IncomingBindingExpressionEnumToI32),
    Field(IncomingBindingExpressionField),
    BindImport(IncomingBindingExpressionBindImport),
}

impl From<IncomingBindingExpressionGet> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionGet) -> Self {
        IncomingBindingExpression::Get(a)
    }
}

impl From<IncomingBindingExpressionAs> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionAs) -> Self {
        IncomingBindingExpression::As(a)
    }
}

impl From<IncomingBindingExpressionAllocUtf8Str> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionAllocUtf8Str) -> Self {
        IncomingBindingExpression::AllocUtf8Str(a)
    }
}

impl From<IncomingBindingExpressionAllocCopy> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionAllocCopy) -> Self {
        IncomingBindingExpression::AllocCopy(a)
    }
}

impl From<IncomingBindingExpressionEnumToI32> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionEnumToI32) -> Self {
        IncomingBindingExpression::EnumToI32(a)
    }
}

impl From<IncomingBindingExpressionField> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionField) -> Self {
        IncomingBindingExpression::Field(a)
    }
}

impl From<IncomingBindingExpressionBindImport> for IncomingBindingExpression {
    fn from(a: IncomingBindingExpressionBindImport) -> Self {
        IncomingBindingExpression::BindImport(a)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionGet {
    pub idx: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionAs {
    pub ty: walrus::ValType,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionAllocUtf8Str {
    pub alloc_func_name: String,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionAllocCopy {
    pub alloc_func_name: String,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionEnumToI32 {
    pub ty: WebidlTypeRef,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionField {
    pub idx: u32,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingBindingExpressionBindImport {
    pub ty: walrus::TypeId,
    pub binding: Id<FunctionBinding>,
    pub expr: Box<IncomingBindingExpression>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebidlTypeRef {
    Id(Id<WebidlCompoundType>),
    Scalar(WebidlScalarType),
}

impl From<WebidlScalarType> for WebidlTypeRef {
    fn from(s: WebidlScalarType) -> Self {
        WebidlTypeRef::Scalar(s)
    }
}

impl From<Id<WebidlCompoundType>> for WebidlTypeRef {
    fn from(i: Id<WebidlCompoundType>) -> Self {
        WebidlTypeRef::Id(i)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebidlScalarType {
    Any,
    Boolean,
    Byte,
    Octet,
    Long,
    UnsignedLong,
    Short,
    UnsignedShort,
    LongLong,
    UnsignedLongLong,
    Float,
    UnrestrictedFloat,
    Double,
    UnrestrictedDouble,
    DomString,
    ByteString,
    UsvString,
    Object,
    Symbol,
    ArrayBuffer,
    DataView,
    Int8Array,
    Int16Array,
    Int32Array,
    Uint8Array,
    Uint16Array,
    Uint32Array,
    Uint8ClampedArray,
    Float32Array,
    Float64Array,
}
