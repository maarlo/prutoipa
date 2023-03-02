use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    descriptor::message_descriptor::{
        field::{Field, FieldModifier, FieldType, ScalarType},
        MessageDescriptor,
    },
    error::PrutoipaBuildError,
    generator::Indent,
};

pub fn generate_message(
    writer: &mut BufWriter<File>,
    name: String,
    message: MessageDescriptor,
) -> Result<(), PrutoipaBuildError> {
    write_head(writer, name)?;

    let res = message
        .get_fields()
        .into_iter()
        .map(|field| write_field(writer, field))
        .collect::<Result<(), PrutoipaBuildError>>();

    write_tail(writer)?;

    res.map(|_| ())
}

fn write_head(writer: &mut BufWriter<File>, name: String) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_08 = Indent(2);

    let lines_to_write = vec![
        format!("{i_00}impl utoipa::ToSchema for {name} {{"),
        format!("{i_04}fn schema() -> utoipa::openapi::schema::Schema {{"),
        format!("{i_08}utoipa::openapi::ObjectBuilder::new()"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_tail(writer: &mut BufWriter<File>) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_12 = Indent(3);

    let lines_to_write = vec![
        format!("{i_12}.into()"),
        format!("{i_04}}}"),
        format!("{i_00}}}"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_field(writer: &mut BufWriter<File>, field: Field) -> Result<(), PrutoipaBuildError> {
    match field.get_field_type() {
        FieldType::Scalar(scalar_type) => write_field_scalar(writer, field, scalar_type),
        field_type => {
            println!(
                "{}",
                PrutoipaBuildError::NotImplementedYet(format!("Field type {field_type:?}"))
            );
            Ok(())
        }
    }
}

fn write_field_scalar(
    writer: &mut BufWriter<File>,
    field: Field,
    scalar_type: ScalarType,
) -> Result<(), PrutoipaBuildError> {
    let i_12 = Indent(3);

    let field_modifier = field.get_field_modifier();
    let field_name = field.get_name();

    if field_modifier == FieldModifier::Repeated {
        println!(
            "{}",
            PrutoipaBuildError::NotImplementedYet("Field modifier repeated".to_string())
        );
    } else {
        let schema_type = scalar_type.get_utoipa_type();
        let property_str = format!("{i_12}.property(\"{field_name}\", utoipa::openapi::Object::with_type(utoipa::openapi::SchemaType::{schema_type}))");
        let required_str = format!("{i_12}.required(\"{field_name}\")");

        writeln!(writer, "{property_str}")?;
        if field_modifier == FieldModifier::Required {
            writeln!(writer, "{required_str}")?;
        }
    }

    Ok(())
}
