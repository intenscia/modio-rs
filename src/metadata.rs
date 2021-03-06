//! Mod metadata KVP interface

use futures::Future as StdFuture;
use hyper::client::connect::Connect;
use url::form_urlencoded;

use types::mods::MetadataMap;
use Future;
use Modio;
use ModioListResponse;
use ModioMessage;
use QueryParams;

pub struct Metadata<C>
where
    C: Clone + Connect + 'static,
{
    modio: Modio<C>,
    game: u32,
    mod_id: u32,
}

impl<C: Clone + Connect + 'static> Metadata<C> {
    pub(crate) fn new(modio: Modio<C>, game: u32, mod_id: u32) -> Self {
        Self {
            modio,
            game,
            mod_id,
        }
    }

    fn path(&self) -> String {
        format!("/games/{}/mods/{}/metadatakvp", self.game, self.mod_id)
    }

    /// Return the metadata key value pairs for a mod that this `Metadata` refers to.
    pub fn get(&self) -> Future<MetadataMap> {
        #[derive(Deserialize)]
        struct KV {
            metakey: String,
            metavalue: String,
        }

        Box::new(
            self.modio
                .get::<ModioListResponse<KV>>(&self.path())
                .map(|list| {
                    let mut map = MetadataMap::new();
                    for kv in list {
                        map.entry(kv.metakey)
                            .or_insert_with(Vec::new)
                            .push(kv.metavalue);
                    }
                    map
                }),
        )
    }

    /// Add metadata for a mod that this `Metadata` refers to.
    pub fn add(&self, metadata: &MetadataMap) -> Future<ModioMessage> {
        self.modio.post(&self.path(), metadata.to_query_params())
    }

    /// Delete metadata for a mod that this `Metadata` refers to.
    pub fn delete(&self, metadata: &MetadataMap) -> Future<()> {
        self.modio.delete(&self.path(), metadata.to_query_params())
    }
}

impl QueryParams for MetadataMap {
    fn to_query_params(&self) -> String {
        let mut ser = form_urlencoded::Serializer::new(String::new());
        for (k, vals) in self.iter() {
            if vals.is_empty() {
                ser.append_pair("metadata[]", k);
            }
            for v in vals {
                ser.append_pair("metadata[]", &format!("{}:{}", k, v));
            }
        }
        ser.finish()
    }
}
