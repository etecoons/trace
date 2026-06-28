pub fn build_arin_address(find_value: &dyn Fn(&str) -> String) -> Vec<String> {
    let street = find_value("Address");
    let city = find_value("City");
    let state = find_value("StateProv");
    let postal = find_value("PostalCode");
    let country = find_value("Country");

    let mut addr_parts = Vec::new();
    if !street.is_empty() {
        addr_parts.push(street);
    }
    let mut city_state_zip = Vec::new();
    if !city.is_empty() {
        city_state_zip.push(city);
    }
    if !state.is_empty() {
        city_state_zip.push(state);
    }
    if !postal.is_empty() {
        city_state_zip.push(postal);
    }
    if !city_state_zip.is_empty() {
        addr_parts.push(city_state_zip.join(", "));
    }
    if !country.is_empty() {
        addr_parts.push(country);
    }
    addr_parts
}

pub fn extract_created(find_value: &dyn Fn(&str) -> String) -> Option<String> {
    let c = find_value("created");
    if !c.is_empty() {
        Some(c)
    } else {
        let r = find_value("RegDate");
        if !r.is_empty() {
            Some(r)
        } else {
            let rd = find_value("reg-date");
            if !rd.is_empty() { Some(rd) } else { None }
        }
    }
}

pub fn extract_last_modified(find_value: &dyn Fn(&str) -> String) -> Option<String> {
    let lm = find_value("last-modified");
    if !lm.is_empty() {
        Some(lm)
    } else {
        let u = find_value("Updated");
        if !u.is_empty() {
            Some(u)
        } else {
            let ch = find_value("changed");
            if !ch.is_empty() { Some(ch) } else { None }
        }
    }
}
