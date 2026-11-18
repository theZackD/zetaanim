use std::collections::BTreeMap;

use crate::{Object, ObjectId};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Scene {
    next_id: u64,
    objects: BTreeMap<ObjectId, Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: Object) -> ObjectId {
        let obj_id = ObjectId(self.next_id);
        self.next_id += 1;
        self.objects.insert(obj_id, object);
        obj_id
    }

    pub fn get(&self, obj_id: ObjectId) -> Option<&Object> {
        self.objects.get(&obj_id)
    }

    pub fn remove(&mut self, obj_id: ObjectId) -> Option<Object> {
        self.objects.remove(&obj_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Construction, Origin};
    use glam::DVec3;

    fn point(name: &str, x: f64, y: f64) -> Object {
        Object {
            name: name.to_string(),
            origin: Origin::Mouse,
            construction: Construction::FreePoint {
                position: DVec3::new(x, y, 0.0),
            },
        }
    }

    #[test]
    fn add_returns_distinct_ids() {
        let mut scene = Scene::new();
        let a = scene.add(point("A", 0.0, 0.0));
        let b = scene.add(point("B", 1.0, 0.0));
        assert_ne!(a, b);
    }

    #[test]
    fn get_returns_added_object() {
        let mut scene = Scene::new();
        let id = scene.add(point("A", 2.0, 3.0));
        assert_eq!(scene.get(id), Some(&point("A", 2.0, 3.0)));
    }

    #[test]
    fn get_unknown_id_returns_none() {
        let mut scene = Scene::new();
        let id = scene.add(point("A", 0.0, 0.0));
        assert_eq!(scene.get(ObjectId(id.0 + 100)), None);
    }

    #[test]
    fn remove_returns_object_and_forgets_it() {
        let mut scene = Scene::new();
        let id = scene.add(point("A", 2.0, 3.0));
        assert_eq!(scene.remove(id), Some(point("A", 2.0, 3.0)));
        assert_eq!(scene.get(id), None);
    }

    #[test]
    fn remove_twice_returns_none() {
        let mut scene = Scene::new();
        let id = scene.add(point("A", 0.0, 0.0));
        scene.remove(id);
        assert_eq!(scene.remove(id), None);
    }

    #[test]
    fn removed_id_is_never_reused() {
        let mut scene = Scene::new();
        let a = scene.add(point("A", 0.0, 0.0));
        scene.remove(a);
        let b = scene.add(point("B", 1.0, 0.0));
        assert_ne!(a, b);
        assert_eq!(scene.get(a), None);
        assert_eq!(scene.get(b).map(|o| o.name.as_str()), Some("B"));
    }

    #[test]
    fn removing_one_object_keeps_the_others() {
        let mut scene = Scene::new();
        let a = scene.add(point("A", 0.0, 0.0));
        let b = scene.add(point("B", 1.0, 0.0));
        scene.remove(a);
        assert_eq!(scene.get(b), Some(&point("B", 1.0, 0.0)));
    }

    #[test]
    fn new_and_default_are_equal() {
        assert_eq!(Scene::new(), Scene::default());
    }
}
