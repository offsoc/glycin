project(
    'glycin-loaders',
    'rust',
    version: '1.0.alpha',
    meson_version: '>=0.57',
)

dependency('gtk4', version: '>= 4.12.0')
dependency('libseccomp', version: '>= 2.5.0')

if 'glycin-heif' in get_option('loaders')
  dependency('libheif', version: '>= 1.17.0')
endif

if 'glycin-jxl' in get_option('loaders')
  dependency('libjxl', version: ['>= 0.8.2', '< 0.10.0'])
endif

if 'glycin-svg' in get_option('loaders')
  dependency('cairo', version: '>= 1.17.0')
endif

i18n = import ('i18n')

compat_version = '1+'

libexecdir = get_option('prefix') / get_option('libexecdir') / meson.project_name() / compat_version
datadir = get_option('prefix') / get_option('datadir')
glycin_datadir = datadir / meson.project_name() / compat_version

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_version()
)

subdir('loaders')