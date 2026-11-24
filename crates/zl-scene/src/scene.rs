use std::collections::{BTreeMap, BTreeSet};

use zl_primitives::Primitive;

use crate::{
    Animation, Camera, Canvas, Construction, EvalContext, Expr, Object, ObjectId, ObjectValue,
    Step, Timeline,
    draw::{LAYER_COUNT, draw_layer, object_primitive},
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Scene {
    next_id: u64,
    objects: BTreeMap<ObjectId, Object>,
    canvas: Canvas,
    camera: Camera,
    timeline: Timeline,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedScene {
    pub values: BTreeMap<ObjectId, ObjectValue>,
    pub blocked: BTreeSet<ObjectId>,
    pub time: f64,
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

    fn evaluation_order(&self) -> (Vec<ObjectId>, BTreeSet<ObjectId>) {
        let mut waiting_for: BTreeMap<ObjectId, usize> = BTreeMap::new();
        let mut dependents: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();

        for id in self.objects.keys() {
            dependents.insert(*id, Vec::new());
        }

        for (id, obj) in &self.objects {
            let mut c = 0;
            for dep in obj.construction.dependencies() {
                if let Some(list) = dependents.get_mut(&dep) {
                    list.push(*id);
                    c += 1;
                }
            }

            waiting_for.insert(*id, c);
        }

        let mut ready: BTreeSet<ObjectId> = BTreeSet::new();
        for (id, c) in &waiting_for {
            if *c == 0 {
                ready.insert(*id);
            }
        }

        let mut order: Vec<ObjectId> = Vec::new();
        while let Some(id) = ready.pop_first() {
            order.push(id);
            for d in &dependents[&id] {
                if let Some(c) = waiting_for.get_mut(d) {
                    *c -= 1;
                    if *c == 0 {
                        ready.insert(*d);
                    }
                }
            }
        }

        let mut blocked: BTreeSet<ObjectId> = BTreeSet::new();
        for (id, c) in waiting_for {
            if c > 0 {
                blocked.insert(id);
            }
        }

        (order, blocked)
    }

    pub fn evaluate(&self, time: f64) -> EvaluatedScene {
        let (order, blocked) = self.evaluation_order();
        let mut ctx = EvalContext {
            time,
            ..EvalContext::default()
        };
        let started = self.timeline.started(time);
        for id in order {
            if let Some(obj) = self.objects.get(&id)
                && let Some(value) = evaluate_object(&obj.construction, id, &started, &ctx)
            {
                ctx.objects.insert(id, value);
            }
        }

        EvaluatedScene {
            values: ctx.objects,
            blocked,
            time,
        }
    }

    pub fn primitives(&self, evaluated: &EvaluatedScene) -> Vec<Primitive> {
        let mut primitives: Vec<Primitive> = Vec::new();
        let mut ctx = EvalContext {
            objects: evaluated.values.clone(),
            variables: Vec::new(),
            time: evaluated.time,
        };

        for layer in 0..LAYER_COUNT {
            for (id, value) in &evaluated.values {
                if let Some(object) = self.objects.get(id)
                    && draw_layer(value, &object.style) == Some(layer)
                    && let Some(primitive) =
                        object_primitive(object, value, &self.camera, &self.canvas, &mut ctx)
                {
                    primitives.push(primitive);
                }
            }
        }
        primitives
    }

    pub fn canvas(&self) -> Canvas {
        self.canvas
    }

    pub fn add_step(&mut self, step: Step) {
        self.timeline.steps.push(step);
    }
}

fn evaluate_object(
    construction: &Construction,
    id: ObjectId,
    started: &[(&Animation, f64)],
    ctx: &EvalContext,
) -> Option<ObjectValue> {
    if !started.iter().any(|(animation, _)| animation.object == id) {
        return construction.evaluate(ctx);
    };

    let mut construction = construction.clone();
    for &(animation, weight) in started {
        if animation.object != id {
            continue;
        }
        let expr = construction.property_mut(animation.property)?;
        let current = expr.evaluate(ctx)?;

        *expr = Expr::Constant(current.interpolate(&animation.to, weight)?);
    }

    construction.evaluate(ctx)
}

#[cfg(test)]
mod tests;
