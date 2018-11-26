use classfile_editor::{
    reader::{ClassReader, ClassReaderFlags},
    ClassAccess, MethodAccess, Type, FieldAccess, AnnotationPrimitive,
    tree::{ClassNode, AnnotationNode, AnnotationValue}
};
use std::{
    collections::HashSet,
    rc::Rc
};
fn get_package_offset(cls: &str) -> usize {
    cls.rfind('/').unwrap_or(0)
}
fn as_java_name(cls: &str) -> String {
    cls.replace('/', ".")
}
fn split_name(cls: &str) -> &str {
    if let Some(ls) = cls.rfind('/') {
        &cls[ls + 1..]
    } else {
        cls
    }
}
fn write_class_access(access: ClassAccess, target: &mut String) {
    if access.contains(ClassAccess::ACC_PUBLIC) {
        target.push_str("public ");
    }
    if access.contains(ClassAccess::ACC_ANNOTATION) {
        target.push_str("@interface ");
    } else if access.contains(ClassAccess::ACC_INTERFACE) {
        target.push_str("interface ");
    } else if access.contains(ClassAccess::ACC_ENUM) {
        target.push_str("enum ");
    } else {
        if access.contains(ClassAccess::ACC_FINAL) {
            target.push_str("final ");
        }
        target.push_str("class ");
    }
}
fn write_method_access(access: MethodAccess, target: &mut String) {
    if access.contains(MethodAccess::ACC_SYNTHETIC) {
        target.push_str("/* synthetic */ ");
    }
    if access.contains(MethodAccess::ACC_BRIDGE) {
        target.push_str("/* bridge */ ");
    }
    if access.contains(MethodAccess::ACC_PUBLIC) {
        target.push_str("public ");
    }
    if access.contains(MethodAccess::ACC_PROTECTED) {
        target.push_str("protected ");
    }
    if access.contains(MethodAccess::ACC_PRIVATE) {
        target.push_str("private ");
    }
    if access.contains(MethodAccess::ACC_STATIC) {
        target.push_str("static ");
    }
    if access.contains(MethodAccess::ACC_FINAL) {
        target.push_str("final ");
    }
    if access.contains(MethodAccess::ACC_SYNCHRONIZED) {
        target.push_str("synchronized ");
    }
    if access.contains(MethodAccess::ACC_NATIVE) {
        target.push_str("native ");
    }
    if access.contains(MethodAccess::ACC_STRICT) {
        target.push_str("strictfp ");
    }
}
fn write_field_access(access: FieldAccess, target: &mut String) {
    if access.contains(FieldAccess::ACC_SYNTHETIC) {
        target.push_str("/* synthetic */ ");
    }
    if access.contains(FieldAccess::ACC_PUBLIC) {
        target.push_str("public ");
    }
    if access.contains(FieldAccess::ACC_PROTECTED) {
        target.push_str("protected ");
    }
    if access.contains(FieldAccess::ACC_PRIVATE) {
        target.push_str("private ");
    }
    if access.contains(FieldAccess::ACC_STATIC) {
        target.push_str("static ");
    }
    if access.contains(FieldAccess::ACC_FINAL) {
        target.push_str("final ");
    }
    if access.contains(FieldAccess::ACC_TRANSIENT) {
        target.push_str("transient ");
    }
    if access.contains(FieldAccess::ACC_VOLATILE) {
        target.push_str("volatile ");
    }
}

fn write_type(mypkg: &str, imports: &mut HashSet<String>, ty: Type, data: &mut String) {
    let dm = ty.dimensions();
    let ty = ty.element_type();
    let dsc = ty.get_descriptor();
    if ty == Type::new("V") {
        data.push_str("void");
    } else if dsc == "Z" {
        data.push_str("boolean");
    } else if dsc == "B" {
        data.push_str("byte");
    } else if dsc == "S" {
        data.push_str("short");
    } else if dsc == "C" {
        data.push_str("char");
    } else if dsc == "I" {
        data.push_str("int");
    } else if dsc == "F" {
        data.push_str("float");
    } else if dsc == "J" {
        data.push_str("long");
    } else if dsc == "D" {
        data.push_str("double");
    } else {
        let raw = import_name(mypkg, imports, &dsc[1..dsc.len() - 1]);
        data.push_str(raw);
    }
    for _ in 0..dm {
        data.push_str("[]");
    }
}
fn needs_import(mypkg: &str, cls: &str) -> bool {
    let pkg = &cls[..get_package_offset(cls)];
    if pkg == "" || pkg == "java/lang" {
        return false;
    }
    pkg != mypkg
}
fn import_name<'a>(mypkg: &str, imports: &mut HashSet<String>, cls: &'a str) -> &'a str {
    if !needs_import(mypkg, cls) {
        return split_name(cls);
    }
    imports.insert(as_java_name(cls));
    split_name(cls)
}
fn write_ann_value(ann: &AnnotationValue, data: &mut String, imports: &mut HashSet<String>, mypkg: &str) {
    match ann {
        AnnotationValue::Primitive(pr) => {
            match pr {
                AnnotationPrimitive::Byte(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Character(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Short(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Integer(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Long(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Float(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Double(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::Boolean(b) => {
                    data.push_str(&format!("{}", b));
                },
                AnnotationPrimitive::String(b) => {
                    data.push_str(&format!("\"{}\"", b));
                },
                AnnotationPrimitive::Type(b) => {
                    let range = 1..b.get_descriptor().len() - 1;
                    data.push_str(import_name(mypkg, imports, &b.get_descriptor()[range]));
                    data.push_str(".class");
                }
            }
        },
        AnnotationValue::Nested(val) => {
            write_annotation(val, data, imports, mypkg);
        },
        AnnotationValue::Array(val) => {
            if val.len() != 1 {
                data.push('{');
            }
            let mut i = 0;
            for v in val {
                write_ann_value(v, data, imports, mypkg);
                if i + 1 != val.len() {
                    data.push_str(", ");
                }
                i += 1;
            }
            if val.len() != 1 {
                data.push('}');
            }
        },
        AnnotationValue::Enum{desc, value} => {
            let range = 1..desc.len() - 1;
            data.push_str(import_name(mypkg, imports, &desc[range]));
            data.push('.');
            data.push_str(value);
        }
    }
}



fn write_annotation(ann: &AnnotationNode, data: &mut String, imports: &mut HashSet<String>, mypkg: &str) {
    data.push('@'); 
    let range = 1..ann.desc.len() - 1;
    data.push_str(import_name(mypkg, imports, &ann.desc[range]));
    data.push('(');
    let mut i = 0;
    if let Some(val) = ann.values.get(&Rc::from("value")) {
        write_ann_value(val, data, imports, mypkg);
        if i + 1 != ann.values.len() {
            data.push_str(", ");
        }
        i += 1;
    }
    for (k, v) in &ann.values {
        if k.as_ref() == "value" {
            continue;
        }
        data.push_str(k);
        data.push_str(" = ");
        write_ann_value(v, data, imports, mypkg);
        if i + 1 != ann.values.len() {
            data.push_str(", ");
        }
        i += 1;
    }
    data.push(')');
} 

pub fn describe(classdata: &[u8]) -> String {
    let rd = ClassReader::new(classdata);
    let mut class = ClassNode::new();
    rd.accept(&mut class, ClassReaderFlags::SKIP_CODE | ClassReaderFlags::SKIP_DEBUG).unwrap();

    let mypkg = &class.name[..get_package_offset(&class.name)];
    let my_name = split_name(&class.name);
    let mut data = String::new();
    let mut imports = HashSet::new();

    for ann in &class.visible_annotations {
        write_annotation(ann, &mut data, &mut imports, mypkg);
        data += "\n";
    }
    for ann in &class.invisible_annotations {
        write_annotation(ann, &mut data, &mut imports, mypkg);
        data += "\n";
    }
    write_class_access(class.access, &mut data);
    data += split_name(&class.name);
    if !class.access.contains(ClassAccess::ACC_ENUM) {
        if let Some(sn) = class.super_name {
            if sn.as_ref() != "java/lang/Object" {
                data += " extends ";
                let raw = import_name(mypkg, &mut imports, &sn);
                data += raw;
            }
        }
    }
    if !class.access.contains(ClassAccess::ACC_ANNOTATION) {
        if class.interfaces.len() > 0 {
            data += " implements ";
            for i in 0..class.interfaces.len() {
                let raw = import_name(mypkg, &mut imports, &class.interfaces[i]);
                data += raw;
                if i + 1 != class.interfaces.len() {
                    data += ", ";
                }
            }
        }
    }
    data += " {\n";
    let en_fields = class.fields.iter().filter(|c| c.access.contains(FieldAccess::ACC_ENUM)).count();
    let mut i = 0;
    for fi in &class.fields {
        if !fi.access.contains(FieldAccess::ACC_ENUM) {
            continue;
        }
        for ann in &fi.visible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        for ann in &fi.invisible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        data += "    ";
        data += &fi.name;
        i += 1;
        if i == en_fields {
            data += ";\n";
        } else {
            data += ",\n";
        }
    }

    for fi in &class.fields {
        if fi.access.contains(FieldAccess::ACC_ENUM) {
            continue;
        }
        for ann in &fi.visible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        for ann in &fi.invisible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        data += "    ";
        write_field_access(fi.access, &mut data);
        write_type(mypkg, &mut imports, Type::new(fi.desc.clone()), &mut data);
        data += " ";
        data += &fi.name;
        data += ";\n";
    }
    for mt in &class.methods {
        for ann in &mt.visible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        for ann in &mt.invisible_annotations {
            data += "    ";
            write_annotation(ann, &mut data, &mut imports, mypkg);
            data += "\n";
        }
        data += "    ";
        if mt.name.as_ref() == "<clinit>" {
            data += "static {};\n";
            continue;
        }
        write_method_access(mt.access, &mut data);
        if mt.name.as_ref() == "<init>" {
            data += &my_name;
        } else {
            write_type(mypkg, &mut imports, Type::new(mt.desc.clone()).return_type(), &mut data);
            data += " ";
            data += &mt.name;
        }
        data += "(";
        let argty = Type::new(mt.desc.clone());
        let mut argty = argty.argument_types();
        let ln = argty.len();
        let mut i = 0;
        for arg in argty.drain(..) {
            write_type(mypkg, &mut imports, arg, &mut data);
            data += &format!(" arg{}", i);
            if i != ln - 1 {
                data += ", ";
            }
            i += 1;
        }
        data += ")";
        if mt.exceptions.len() != 0 {
            data += " throws ";
        }
        for i in 0..mt.exceptions.len() {
            let raw = import_name(mypkg, &mut imports, &mt.exceptions[i]);
            data += raw;
            if i != mt.exceptions.len() - 1 {
                data += ", ";
            }
        }
        if let Some(ref an) = mt.annotation_default {
            data += " default ";
            write_ann_value(an.values.get(&Rc::from("")).unwrap(), &mut data, &mut imports, mypkg);
        }
        data += ";\n";
    }

    data += "}\n";
    let mut header = "package ".to_owned() + &mypkg.replace('/', ".") + ";\n\n";
    for imp in &imports {
        header += "import ";
        header += imp;
        header += ";\n";
    }
    header += "\n";
    data = header + &data;
    data
}


