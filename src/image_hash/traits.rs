use imagehash::Hash;
use image;

pub trait Hasher {
    fn hash(&self, image: &image::DynamicImage) -> Hash;
}

impl Hasher for imagehash::PerceptualHash {
    fn hash(&self, image: &image::DynamicImage) -> Hash {
        self.hash(image)
    }
}

impl Hasher for imagehash::DifferenceHash {
    fn hash(&self, image: &image::DynamicImage) -> Hash {
        self.hash(image)
    }
}

impl Hasher for imagehash::AverageHash {
    fn hash(&self, image: &image::DynamicImage) -> Hash {
        self.hash(image)
    }
}