use std::io::Write;

use crate::{
    descriptor::message_descriptor::{
        field::{Field, FieldModifier, FieldType, ScalarType},
        MessageDescriptor,
    },
    error::PrutoipaBuildError,
    generator::Indent,
};

pub fn generate_message<W: Write>(
    writer: &mut W,
    package_name: String,
    name: String,
    message: MessageDescriptor,
) -> Result<(), PrutoipaBuildError> {
    write_head(writer, package_name.clone(), name)?;

    let res = message
        .get_fields()
        .into_iter()
        .map(|field| write_field(writer, package_name.clone(), field))
        .collect::<Result<(), PrutoipaBuildError>>();

    write_tail(writer)?;

    res.map(|_| ())
}

fn write_head<W: Write>(
    writer: &mut W,
    package_name: String,
    name: String,
) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_08 = Indent(2);
    let i_12 = Indent(3);

    let lines_to_write = vec![
        format!("{i_00}impl<'__s> utoipa::ToSchema<'__s> for {name} {{"),
        format!("{i_04}fn schema() -> (&'__s str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {{"),
        format!("{i_08}("),
        format!("{i_12}\"{package_name}::{name}\","),
        format!("{i_12}utoipa::openapi::ObjectBuilder::new()"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_tail<W: Write>(writer: &mut W) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_08 = Indent(2);
    let i_16 = Indent(4);

    let lines_to_write = vec![
        format!("{i_16}.into()"),
        format!("{i_08})"),
        format!("{i_04}}}"),
        format!("{i_00}}}"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_field<W: Write>(
    writer: &mut W,
    package_name: String,
    field: Field,
) -> Result<(), PrutoipaBuildError> {
    match field.get_field_type() {
        FieldType::Scalar(scalar_type) => write_field_scalar(writer, field, scalar_type),
        FieldType::Object {
            package,
            descriptor,
        } => write_field_object(writer, field, package_name, package, descriptor),
    }
}

fn write_field_scalar<W: Write>(
    writer: &mut W,
    field: Field,
    scalar_type: ScalarType,
) -> Result<(), PrutoipaBuildError> {
    let i_16 = Indent(4);
    let i_20 = Indent(5);

    let field_modifier = field.get_field_modifier();
    let field_name = field.get_name();

    //
    let mut property_str = Vec::<String>::new();
    property_str.push(format!("{i_16}.property("));
    property_str.push(format!("{i_20}\"{field_name}\","));

    if field_modifier == FieldModifier::Repeated {
        property_str.push(format!(
            "{i_20}utoipa::openapi::ArrayBuilder::from(utoipa::openapi::Array::new("
        ));

        property_str.append(&mut get_field_scalar_component(6, scalar_type));

        property_str.push(format!("{i_20}))"));
    } else {
        property_str.append(&mut get_field_scalar_component(5, scalar_type));
    }

    property_str.push(format!("{i_16})"));

    for property_str_line in property_str {
        writeln!(writer, "{property_str_line}")?;
    }

    //
    if field_modifier == FieldModifier::Required {
        let required_str = format!("{i_16}.required(\"{field_name}\")");
        writeln!(writer, "{required_str}")?;
    }

    Ok(())
}

fn get_field_scalar_component(base_indent: usize, scalar_type: ScalarType) -> Vec<String> {
    let mut property_str = Vec::<String>::new();

    let i_00 = Indent(base_indent);
    let i_04 = Indent(base_indent + 1);
    let i_08 = Indent(base_indent + 2);

    let schema_type = scalar_type.get_utoipa_type();
    let schema_format = scalar_type.get_utoipa_format();

    //
    property_str.push(format!("{i_00}utoipa::openapi::ObjectBuilder::new()"));
    property_str.push(format!(
        "{i_04}.schema_type(utoipa::openapi::SchemaType::{schema_type})"
    ));

    if let Some(known_format) = schema_format {
        property_str.push(format!(
            "{i_04}.format(Some(utoipa::openapi::SchemaFormat::KnownFormat("
        ));
        property_str.push(format!(
            "{i_08}utoipa::openapi::KnownFormat::{known_format}"
        ));
        property_str.push(format!("{i_04})))"));
    }

    property_str
}

fn write_field_object<W: Write>(
    writer: &mut W,
    field: Field,
    current_package: String,
    field_package: String,
    field_descriptor: String,
) -> Result<(), PrutoipaBuildError> {
    let i_16 = Indent(4);
    let i_20 = Indent(5);
    let i_24 = Indent(6);

    let field_modifier = field.get_field_modifier();
    let field_name = field.get_name();

    //
    let mut property_str = Vec::<String>::new();
    property_str.push(format!("{i_16}.property("));
    property_str.push(format!("{i_20}\"{field_name}\","));

    if field_modifier == FieldModifier::Repeated {
        property_str.push(format!(
            "{i_20}utoipa::openapi::ArrayBuilder::from(utoipa::openapi::Array::new("
        ));

        property_str.push(get_field_object_component(
            i_24,
            current_package,
            field_package,
            field_descriptor,
        ));

        property_str.push(format!("{i_20}))"));
    } else {
        property_str.push(get_field_object_component(
            i_20,
            current_package,
            field_package,
            field_descriptor,
        ));
    }

    property_str.push(format!("{i_16})"));

    for property_str_line in property_str {
        writeln!(writer, "{property_str_line}")?;
    }

    //
    if field_modifier == FieldModifier::Required {
        let required_str = format!("{i_16}.required(\"{field_name}\")");
        writeln!(writer, "{required_str}")?;
    }

    Ok(())
}

fn get_field_object_component(
    indent: Indent,
    current_package: String,
    field_package: String,
    field_descriptor: String,
) -> String {
    if current_package == field_package {
        format!("{indent}{field_descriptor}::schema()")
    } else {
        format!("{indent}super::{field_package}::{field_descriptor}::schema()")
    }
}
