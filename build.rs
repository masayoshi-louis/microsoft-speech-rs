fn main() {
    println!("cargo:rustc-link-lib=dylib=Microsoft.CognitiveServices.Speech.core");
    println!("cargo:rustc-link-search=native=./lib");
}
