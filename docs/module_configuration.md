# Module Configuration

## TODO

## Configuration Example

Content of file `./module.anc.ason`:

```json5
{
    name: "hello"
    version: "1.0.0"
    edition: "2025"

    // TODO
    seal: false

    properties: [
        // Declares properties and there default values for
        // used by the current program (module).
        // This value of the property declared here can be
        // read in the program's souce code by the macro `prop!(...)`.
        //
        //
        // Strings can interoperate with properties using
        // the placeholder `{name}`
        //
        // e.g.
        // version: "{logger_version}"

        "enable_abc": prop::bool(true)                      // bool/flag
        "enable_xyz": prop::bool(true)                      // bool/flag
        "enable_all": prop::set(false, [                    // bool set
            "enable_abc"
            "enable_xyz"
            ])
        "enable_logger": prop::eval(                        // evaluation
            "enable_abc && not(enable_xyz)")
        "logger_version": prop::string("1.0.1")             // string value
        "bits": prop::number(32)                            // number value
    ]
    modules: [                                              // dependencies
        "std": module::runtime
        "digest": module::share({
            version: "1.0"
            parameters: [                                   // pass values to module
               "enable_sha2": param::bool(true)             // bool/flag
               "enable_md5": param::bool(false)             // bool/flag
               "enable_foo": param::eval("not(enable_md5)") // evaluation
               "bits": param::prop("bits")                  // inherited
               "enable_abc": param::prop("enable_abc")      // inherited
            ]
            repository: "custom"
        })
        "logger": module::share({
            version: "{logger_version}"                     // string interpolation
            condition: cond::is_true("enable_logger")
        })
    ]
    libraries: [                                            // dependent libraries
        "foo": library::remote({
            url: "https://github.com/..."
            revision: "v1.0.1"
            path: "/lib/libfoo.so.1"
        })
        "bar": library::system("libbar.so.1")
    ]
}
```
