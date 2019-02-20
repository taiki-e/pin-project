# Unreleased

# 0.3.0 - 2019-02-20

* Remove `unsafe_fields` attribute.

* Remove `unsafe_variants` attribute.

# 0.2.2 - 2019-02-20

* Fix a bug that generates incorrect code for the some structures with trait bounds on type generics.

# 0.2.1 - 2019-02-20

* Fix a bug that generates incorrect code for the structures with where clause and associated type fields.

# 0.2.0 - 2019-02-11

* Make `unsafe_fields` optional.

* Improve documentation.

# 0.1.8 - 2019-02-02

* Add the feature to create projected enums to `unsafe_project`.

* Add `project` attribute to support pattern matching.

# 0.1.7 - 2019-01-19

* Fix documentation.

# 0.1.6 - 2019-01-19

* `unsafe_fields` can now opt-out.

* Add `unsafe_variants` attribute. This attribute is available if pin-project is built with the "unsafe_variants" feature.

# 0.1.5 - 2019-01-17

* Add support for tuple struct to `unsafe_project`.

# 0.1.4 - 2019-01-12

* Add options for automatically implementing `Unpin` to both `unsafe_project` and `unsafe_fields`.

# 0.1.3 - 2019-01-11

* Fix dependencies.

* Add `unsafe_fields` attribute.

# 0.1.2 - 2019-01-09

* Improve documentation.

# 0.1.1 - 2019-01-08

* Rename from `unsafe_pin_project` to `unsafe_project`.

# 0.1.0 - 2019-01-08

Initial release
