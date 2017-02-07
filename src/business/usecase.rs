use entities::*;
use super::date_restriction::DateRestriction;

struct MockRepository;

impl MockRepository {
    fn GetEntries() -> Vec<Entry> {
        unimplemented!();
    }

    fn GetTags() -> Vec<Tag> {
        unimplemented!();
    }

    fn GetTriples() -> Vec<SentenceTriple> {
        unimplemented!();
    }
}

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

pub fn search_by_tags(tags : &Vec<String>, up_to : DateRestriction) -> Vec<Entry> {
    unimplemented!();

    let tag_ids = get_tag_ids_by_tags(tags);
    let ids = get_associated_entry_ids_of_tags(&tag_ids);
    let entries = get_entries_by_ids(&ids, up_to);

    entries
}

pub fn get_tag_ids_by_tags(tags : &Vec<String>) -> Vec<String> {
    unimplemented!();
}

pub fn get_associated_entry_ids_of_tags(tag_ids : &Vec<String>) -> Vec<String> {
    unimplemented!();
}

pub fn get_entries_by_ids(ids : &Vec<String>, up_to : DateRestriction) -> Vec<Entry> {
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


use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Repo;
use super::validate::Validate;
use uuid::Uuid;

type Result<T> = result::Result<T,Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewEntry {
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
    license     : String,
}

pub fn create_new_entry<R: Repo<Entry>>(r: &mut R, e: NewEntry) -> Result<String>
 {
    let e = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        title       :  e.title,      
        description :  e.description,
        lat         :  e.lat,        
        lng         :  e.lng,        
        street      :  e.street,     
        zip         :  e.zip,        
        city        :  e.city,       
        country     :  e.country,    
        email       :  e.email,      
        telephone   :  e.telephone,  
        homepage    :  e.homepage,   
        categories  :  e.categories, 
        tags        :  e.tags,
        license     :  Some(e.license)    
    };
    e.validate()?;
    r.create(&e)?;
    Ok(e.id)
}

pub struct NewTag {
    name : String
}

pub fn create_new_tag<R: Repo<Tag>>(r: &mut R, t: NewTag) -> Result<String> {
    let t = Tag {
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        name        :  t.name
    };
    r.create(&t)?;
    Ok(t.id)
}

////////////////
// TESTS
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: empty vec of tags
    // OUTPUT: empty vec of entities
    fn empty_search_by_tags()
    {
        assert!(false);
    }

    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: vec of one tag (existing)
    // OUTPUT: vec of associated entries
    // ASSERT: only the fitting entries are given back
    fn search_by_one_tag()
    {
        assert!(false);
    }

    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: vec of one tag (undefined)
    // OUTPUT: vec of associated entries
    // ASSERT: output should be empty
    fn search_by_undefined_tag()
    {
        assert!(false);
    }

    #[test]
    // ENVIRONMENT: tags, entries, some links from tags to entries
    // INPUT: vec of tags, date restriction
    // OUTPUT: vec of associated entries, but only up to the given date
    fn search_by_tags_with_date_restriction()
    {
        assert!(false);
    }

    type RepoResult<T> = result::Result<T, RepoError>;

    struct MockRepo<T> {
        objects: Vec<T>,
    }

    impl Repo<Entry> for MockRepo<Entry> {
        type Id = String;

        fn get(&self, id: Self::Id) -> RepoResult<Entry> {
            match self.objects.iter().find(|x| x.id == id) {
                Some(x) => Ok(x.clone()),
                None => Err(RepoError::NotFound),
            }
        }

        fn all(&self) -> RepoResult<Vec<Entry>> {
            Ok(self.objects.clone())
        }

        fn create(&mut self, e: &Entry) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }

        fn update(&mut self, e: &Entry) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }
    }



    impl Repo<Tag> for MockRepo<Tag> {
        type Id = String;

        fn get(&self, id: Self::Id) -> RepoResult<Tag> {
            match self.objects.iter().find(|x| x.id == id) {
                Some(x) => Ok(x.clone()),
                None => Err(RepoError::NotFound),
            }
        }

        fn all(&self) -> RepoResult<Vec<Tag>> {
            Ok(self.objects.clone())
        }

        fn create(&mut self, e: &Tag) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }

        fn update(&mut self, e: &Tag) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }
    }




    #[test]
    fn create_new_valid_entry() {
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into()
        };
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        let now = UTC::now();
        let id = create_new_entry(&mut mock_db, x).unwrap();
        assert!(Uuid::parse_str(&id).is_ok());
        assert_eq!(mock_db.objects.len(),1);
        let x = &mock_db.objects[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.description, "bar");
        assert_eq!(x.version, 0);
        assert!(x.created as i64 >= now.timestamp());
        assert!(Uuid::parse_str(&x.id).is_ok());
        assert_eq!(x.id, id);
    }

    #[test]
    fn create_entry_with_invalid_email() {
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : Some("fooo-not-ok".into()),
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into()
        };
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        assert!(create_new_entry(&mut mock_db, x).is_err());
    }
}
