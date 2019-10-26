use libc;

use std::mem;
use std::slice;
use std::str;

use markup5ever::QualName;

use crate::util::{make_qual_name, opt_utf8_cstr, utf8_cstr};

pub struct PropertyBag<'a>(Vec<(QualName, &'a str)>);

pub struct PropertyBagIter<'a>(slice::Iter<'a, (QualName, &'a str)>);

impl<'a> PropertyBag<'a> {
    /// Creates an iterable `PropertyBag` from a C array of borrowed C strings.
    ///
    /// With libxml2's SAX parser, the caller's startElementNsSAX2Func
    /// callback gets passed a `xmlChar **` for attributes, which
    /// comes in groups of /// (localname/prefix/URI/value_start/value_end).
    /// In those, localname/prefix/URI are NUL-terminated strings;
    /// value_start and value_end point to the start-inclusive and
    /// end-exclusive bytes in the attribute's value.
    ///
    /// This function is unsafe because the caller must guarantee the following:
    ///
    /// * `attrs` is a valid pointer, with (n_attributes * 5) elements.
    ///
    /// * All strings are valid UTF-8.
    ///
    /// The lifetime of the `PropertyBag` should be considered the same as the lifetime of the
    /// `attrs` array, as the property bag does not copy the strings - it directly stores pointers
    /// into that array's strings.
    pub unsafe fn new_from_xml2_attributes(
        n_attributes: usize,
        attrs: *const *const libc::c_char,
    ) -> PropertyBag<'a> {
        let mut array = Vec::new();

        if n_attributes > 0 && !attrs.is_null() {
            let attrs = slice::from_raw_parts(attrs, n_attributes * 5);

            let mut i = 0;
            while i < n_attributes * 5 {
                let localname = attrs[i];
                let prefix = attrs[i + 1];
                let uri = attrs[i + 2];
                let value_start = attrs[i + 3];
                let value_end = attrs[i + 4];

                assert!(!localname.is_null());

                let prefix = opt_utf8_cstr(prefix);
                let uri = opt_utf8_cstr(uri);
                let localname = utf8_cstr(localname);
                let qual_name = make_qual_name(prefix, uri, localname);

                if !value_start.is_null() && !value_end.is_null() {
                    assert!(value_end >= value_start);

                    // FIXME: ptr::offset_from() is nightly-only.
                    // We'll do the computation of the length by hand.
                    let start: usize = mem::transmute(value_start);
                    let end: usize = mem::transmute(value_end);
                    let len = end - start;

                    let value_slice = slice::from_raw_parts(value_start as *const u8, len);
                    let value_str = str::from_utf8_unchecked(value_slice);

                    array.push((qual_name, value_str));
                }

                i += 5;
            }
        }

        PropertyBag(array)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> PropertyBagIter<'_> {
        PropertyBagIter(self.0.iter())
    }
}

impl<'a> Iterator for PropertyBagIter<'a> {
    type Item = (QualName, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(a, v)| (a.clone(), v.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markup5ever::{local_name, namespace_url, ns, LocalName};
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn empty_property_bag() {
        let map = unsafe { PropertyBag::new_from_xml2_attributes(0, ptr::null()) };
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn property_bag_with_namespaces() {
        let attrs = [
            (CString::new("rx").unwrap(),
             CString::new("svg").unwrap(),
             CString::new("http://www.w3.org/2000/svg").unwrap(),
             CString::new("1").unwrap()),

            (CString::new("ry").unwrap(),
             CString::new("svg").unwrap(),
             CString::new("http://www.w3.org/2000/svg").unwrap(),
             CString::new("2").unwrap()),

            (CString::new("empty").unwrap(),
             CString::new("svg").unwrap(),
             CString::new("http://www.w3.org/2000/svg").unwrap(),
             CString::new("").unwrap()),
        ];

        let mut v: Vec<*const libc::c_char> = Vec::new();

        for (localname, prefix, uri, val) in &attrs {
            v.push(localname.as_ptr() as *const libc::c_char);
            v.push(prefix.as_ptr() as *const libc::c_char);
            v.push(uri.as_ptr() as *const libc::c_char);

            let val_start = val.as_ptr() as *const libc::c_char;
            let val_end = unsafe { val_start.offset(val.as_bytes().len() as isize) };
            v.push(val_start); // value_start
            v.push(val_end); // value_end
        }

        let pbag = unsafe { PropertyBag::new_from_xml2_attributes(3, v.as_ptr()) };

        let mut had_rx: bool = false;
        let mut had_ry: bool = false;
        let mut had_empty: bool = false;

        for (a, v) in pbag.iter() {
            assert_eq!(a.prefix.as_ref().unwrap(), "svg");
            assert_eq!(a.ns, ns!(svg));

            match a.local {
                local_name!("rx") => {
                    assert!(v == "1");
                    had_rx = true;
                }
                local_name!("ry") => {
                    assert!(v == "2");
                    had_ry = true;
                }
                ref n if *n == LocalName::from("empty") => {
                    assert!(v == "");
                    had_empty = true;
                }
                _ => unreachable!(),
            }
        }

        assert!(had_rx);
        assert!(had_ry);
        assert!(had_empty);
    }
}
