project(
    'glycin-loaders',
    'rust',
    version: '0.1.beta.2',
    meson_version: '>=0.57',
)

dependency('gtk4', version: '>= 4.11.2')

if 'glycin-heif' in get_option('loaders')
  dependency('libheif', version: '>= 1.14.2')
endif

i18n = import ('i18n')

compat_version = '0+'

libexecdir = get_option('prefix') / get_option('libexecdir') / meson.project_name() / compat_version
datadir = get_option('prefix') / get_option('datadir') / meson.project_name() / compat_version

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_version()
)

subdir('loaders')