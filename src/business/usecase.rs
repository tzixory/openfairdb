use entities::*;
use super::date_restriction::DateRestriction;

////////////////
// USE CASE: user requests an entry
//
// What should happen:
// * assume the user has already the base ID (e.g. from a research by name
//   or by tag)
// * get the entry base of that ID
// * get the newest entry of that ID
// * get the list of tags that links to that ID, updated to the newest state
//   (respecting all additions and deletions of tags)
//
// * return the entry and the list of tags

////////
// sub case: no date restriction, return head entries

//
////////

////////
// sub case: date restriction given, return latest entries before date

//
////////

//
// USE CASE: user requests an entry (head entry, no date restriction)
////////////////



////////////////
// USE CASE: user researches a tag
//
// What should happen:
// * assume the user only knows the keyword he wants to research
// * find the ID associated with the keyword
// * get a list of all entries that are linked with that tag
// ** i.e. first, get all IDs associated with the tag
// ** then, get the entries associated with the IDs
// ** (future) follow equivalence and sub-class links
//
// * return the newest state of each entry

pub fn research_a_keyword(keyword : String, up_to : DateRestriction) -> Vec<Entry> {
    unimplemented!();

    let tag_id = get_tag_id_by_keyword(keyword);
    let ids = get_associated_entry_ids_of_tag(tag_id);
    let entries = get_entries_by_ids(ids, up_to);

    entries
}

pub fn get_tag_id_by_keyword(keyword : String) -> String {
    unimplemented!();
}

pub fn get_associated_entry_ids_of_tag(tag_id : String) -> Vec<String> {
    unimplemented!();
}

pub fn get_entries_by_ids(ids : Vec<String>, up_to : DateRestriction) -> Vec<Entry> {
    unimplemented!();
}



////////
// sub case: no date restriction, return head entries

//
////////

////////
// sub case: date restriction given, return latest entries before date

//
////////

//
// USE CASE: user researches a tag
////////////////



////////////////
// USE CASE: (future) onthological researches
// 
// What should happen:
// * the user researches sub-class / super-class / equivalence / similarity
//   relations to a keyword
// * return all sub-class tags / all super-class tags / all equivalent tags /
//   all direct similarities

////////
// sub-case: sub-classes

//
////////

////////
// sub-case: super-classes

//
////////

////////
// sub-case: equivalences

//
////////

////////
// sub-case: similarities

//
////////

// USE CASE: (future) onthological researches
////////////////
