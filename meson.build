project('ianny', 'rust',
  version : '0.1.0-alpha.0',
  license : 'GPL3',
  default_options : ['warning_level=2'])

application_id = 'io.github.zer0_x.ianny'

# Get options
profile = get_option('profile')

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')

# Check for deps
dbus_dep = dependency('dbus-1', version : '>=1.6.0')

desktop_utils = find_program('desktop-file-validate', required: false)
cargo = find_program('cargo', required: true)

# Desktop file
desktop_file ='@0@.desktop'.format(application_id)

if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

# Build summaries
summary({
    'Profile': get_option('profile'),
  },
  section: 'Build options',
)

# Other meson.build files
subdir('src/')


# Install .desktop files
install_data(
  desktop_file,
  install_dir: 'share/applications'
)

install_data(
  desktop_file,
  install_dir: '/etc/xdg/autostart/'
)
