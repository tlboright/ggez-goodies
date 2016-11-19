// Useful references:
// https://www.reddit.com/r/gamedev/comments/2vlypg/i_made_a_html5_particle_engine/
// https://www.reddit.com/r/gamedev/comments/135w5u/version_five_of_my_2d_particle_system_is_complete/
// https://www.reddit.com/r/gamedev/comments/13ksu3/article_on_particle_systems_and_an_online_cocos2d/
// Unity3D's particle system
// Cocos2d's plist file format
// Oh, love2d's particle system parameters, derp.


use std::f64;

extern crate rand;
use self::rand::Rng;
extern crate nalgebra as na;

use ggez::{GameResult, Context};
use ggez::graphics;

type Point2 = na::Point2<f64>;
type Vector2 = na::Vector2<f64>;

struct Particle {
    pos: Point2,
    vel: Vector2,
    age: f64,
}

/// A trait that defines a way to do some sort of
/// lerp or easing function on a type.
trait Interpable {
    fn interp(&self, t: f64) -> Self;
}

/// A structure that represents a transition between
/// set properties, with multiple potential defined points.
/// So for instance you could use Transition<Color> and define
/// a transition of colors from red to orange to grey to do smoke.
/// You could also use Transition<f64> to just represent a size
/// curve.
/// So really this is a general-purpose easing type thing...
/// It assumes that all time values range from 0 to 1, currently.
/// Though we could fix that just by having or finding some kind of
/// scaling factor... hmmmm.  Nah, that should be external to the
/// transition.
struct Transition<T: Interpable> {
    breakpoints: Vec<(f64, T)>,
}

impl<T: Interpable> Transition<T> {
    /// Add a new breakpoint to the transition
    /// at time 0 < t < 1
    fn add(&mut self, t: f64, val: T) {}
}

enum StartParam<T> {
    Fixed(T),
    UniformRange(T, T),
}

use self::rand::distributions::Sample;

impl<T> Sample<f64> for StartParam<T> {
    fn sample<R: Rng>(&mut self, rng: &mut R) -> f64 {
        let rand::Open01(val) = rand::random::<rand::Open01<f64>>();
        val
    }
}


impl StartParam<f64> {
    fn get_value(self) -> f64 {
        match self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(ref low, ref high) => {
                //let mut rng = rand::thread_rng();
                //rng.gen()
                rand::random::<StartParam<f64>>()
            }
        }
    }
}


impl StartParam<f32> {
    fn get_value(self) -> f32 {
        match self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(ref low, ref high) => {
                let mut rng = rand::thread_rng();
                rng.gen()
            }
        }
    }
}



// Properties particles should have:
// Age, position, velocity

// Properties particle systems should have:
// color, inc. opacity
// texture (perhaps sprite?), multiplied by color
// size
// gravity
// fade rate/color transitions
// max lifespan
// speed
// xvel, yvel
// shape???
// Gravity???
// Glow???
// x/y bounds (delete particles that go further than this)
// floor and ceiling?  (particles bounce off of these)
//
// Per love2d, which appears to cover all the basics and more:
// area spread (uniform, normal)
// * buffer size (number of particles)
// * linear acceleration (general case of gravity)
// color (of image)
// colors (of non-image particle)
// direction
// emission rate (constant, burst)
// emitter lifetime
// image
// insert mode (where particles are inserted; top, bottom, random)
// lifetime
// linear damping
// particle lifetime (min, max)
// position of emitter
// quads (series of images to use as sprites)
// radial acceeleration
// rotation
// size variations/sizes
// set speed
// spin, spin variation
// spread
// tangential acceleration
//
// Honestly having general purpose "create" and "update" traits
// would abstract out a lot of this, and then we just define
// the basics.
//
// It would also be very nice to be able to have a particle system
// calculate in is own relative coordinate space OR world absolute space.
// Though if the user defines their own worldspace coordinate system
// that could get a bit sticky.  :/

impl Particle {
    fn new(pos: Point2, vel: Vector2) -> Self {
        Particle {
            pos: pos,
            vel: vel,
            age: 0.0,
        }
    }
}

// This probably isn't actually needed as a separate type, 
// at least at this point,
// but it makes things clearer for the moment...  Hmm.
pub struct ParticleSystemBuilder {
    system: ParticleSystem,
}

impl ParticleSystemBuilder {
    pub fn new() -> Self {
        let system = ParticleSystem::new();
        ParticleSystemBuilder {
            system: system
        }
    }
    pub fn build(self) -> ParticleSystem {
        self.system
    }

    /// Set maximum number of particles.
    pub fn count(mut self, count: usize) -> Self {
        self.system.max_particles = count;
        self.system.particles.reserve_exact(count);
        self
    }

    pub fn lifetime(mut self, time: f64) -> Self {
        self.system.max_life = time;
        self

    }

    pub fn acceleration(mut self, accel: Vector2) -> Self {
        self.system.acceleration = accel;
        self
    }
}


pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
    max_life: f64,
    acceleration: Vector2,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem { 
            particles: Vec::new(), 
            max_particles: 0 ,
            max_life: f64::INFINITY,
            acceleration: Vector2::new(0.0, 0.0),
        }
    }

    pub fn emit(&mut self) {
        let pos = Point2::new(0.0, 0.0);
        let vec = Vector2::new(10.0, 10.0);
        let newparticle = Particle::new(pos, vec);
        self.add_particle(newparticle);
    }

    pub fn update(&mut self, dt: f64) {
        for mut p in self.particles.iter_mut() {
            p.vel += self.acceleration * dt;
            p.pos += p.vel * dt;
            p.age += dt;
        }

        // Gotta make borrowck happy by not referring
        // to self in the same closure twice.
        let max_life = self.max_life;
        self.particles.retain(|p| p.age < max_life);
    }

    fn calc_particle_size(&self, idx: usize) -> u32 {
        5
    }

    /// Adds a new particle to the system, if it would
    /// not exceed the max number of particles in the system.
    fn add_particle(&mut self, p: Particle) {
        if self.particles.len() <= self.max_particles {
            self.particles.push(p);
        }
    }
}

impl graphics::Drawable for ParticleSystem {
    fn draw_ex(&self,
               context: &mut Context,
               src: Option<graphics::Rect>,
               dst: Option<graphics::Rect>,
               angle: f64,
               center: Option<graphics::Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()> {
        // BUGGO: Width and height here should be the max bounds of the
        // particle system...?
        // It'd be consistent with our drawing API, but would require
        // finding the bounds of all particles on every tick, which is
        // expensive(ish).
        // Maybe we can make it an x and y scale?  Hmm.
        let dst_rect = dst.unwrap_or(graphics::Rect::new(0, 0, 0, 0));
        for (i,p) in self.particles.iter().enumerate() {
            let p_size = self.calc_particle_size(i);
            let rect = graphics::Rect::new(dst_rect.x() + p.pos.x as i32,
                                           dst_rect.y() + p.pos.y as i32,
                                           p_size,
                                           p_size);
            graphics::rectangle(context, graphics::DrawMode::Fill, rect)?;
        }
        Ok(())
    }
}
