import os
import sys
from pathlib import Path
from datetime import timedelta
import pymongo


# Build paths inside the project like this: BASE_DIR / 'subdir'.
BASE_DIR = Path(__file__).resolve().parent.parent

from dotenv import load_dotenv
load_dotenv(os.path.join(BASE_DIR, '.env'))


# SECURITY WARNING: keep the secret key used in production secret!
SECRET_KEY = os.environ.get('SECRET_KEY', 'django-insecure-key-for-development-only')

# SECURITY WARNING: don't run with debug turned on in production!
DEBUG = os.environ.get('DEBUG', 'True') == 'True'

ALLOWED_HOSTS = os.environ.get('ALLOWED_HOSTS', '*').split(',')

# Application definition
INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',

    # Third-party apps
    'rest_framework',
    'rest_framework_simplejwt',
    'drf_yasg',
    'django_filters',
    'corsheaders',
    'django_prometheus',

    # Local apps
    'admin_users',
    'authentication',
    'nas',
    'users',
    'accounting',
    'radius',
    'radsec',
    'shared'
]

MIDDLEWARE = [
    # Prometheus middleware should be at the top
    'django_prometheus.middleware.PrometheusBeforeMiddleware',

    'django.middleware.security.SecurityMiddleware',
    # Whitenoise middleware should be placed after SecurityMiddleware
    'whitenoise.middleware.WhiteNoiseMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    # CORS middleware should be placed before CommonMiddleware
    'corsheaders.middleware.CorsMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',

    # Prometheus middleware should also be at the bottom
    'django_prometheus.middleware.PrometheusAfterMiddleware',
]

ROOT_URLCONF = 'core.urls'

TEMPLATES = [
    {
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
        'DIRS': [],
        'APP_DIRS': True,
        'OPTIONS': {
            'context_processors': [
                'django.template.context_processors.debug',
                'django.template.context_processors.request',
                'django.contrib.auth.context_processors.auth',
                'django.contrib.messages.context_processors.messages',
            ],
        },
    },
]

WSGI_APPLICATION = 'core.wsgi.application'

# Database
DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.postgresql',
        'NAME': os.environ.get('DB_NAME', 'postgres'),
        'USER': os.environ.get('DB_USER', 'postgres'),
        'PASSWORD': os.environ.get('DB_PASSWORD', 'postgres'),
        'HOST': os.environ.get('DB_HOST', 'db'),
        'PORT': os.environ.get('DB_PORT', '5432'),
    }
}

# Password validation
AUTH_PASSWORD_VALIDATORS = [
    {
        'NAME': 'django.contrib.auth.password_validation.UserAttributeSimilarityValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.MinimumLengthValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.CommonPasswordValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.NumericPasswordValidator',
    },
]

# Internationalization
LANGUAGE_CODE = 'en-us'
TIME_ZONE = 'UTC'
USE_I18N = True
USE_TZ = True

# Static files (CSS, JavaScript, Images)
STATIC_URL = '/api/static/'
STATIC_ROOT = os.path.join(BASE_DIR, 'static')

# Ensure static files are properly served in both development and production
STATICFILES_DIRS = []
if DEBUG:
    # In development, Django will serve static files directly
    STATICFILES_FINDERS = [
        'django.contrib.staticfiles.finders.FileSystemFinder',
        'django.contrib.staticfiles.finders.AppDirectoriesFinder',
    ]
else:
    # In production, use WhiteNoise for static files
    STATICFILES_STORAGE = 'whitenoise.storage.CompressedManifestStaticFilesStorage'

# WhiteNoise configuration
WHITENOISE_MAX_AGE = 31536000  # 1 year in seconds
WHITENOISE_AUTOREFRESH = DEBUG

# Default primary key field type
DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'

# Custom user model
AUTH_USER_MODEL = 'admin_users.AdminUser'

# Custom authentication backend
AUTHENTICATION_BACKENDS = [
    'admin_users.backends.EmailBackend',
    'django.contrib.auth.backends.ModelBackend',
]

# REST Framework settings
REST_FRAMEWORK = {
    'DEFAULT_AUTHENTICATION_CLASSES': (
        'rest_framework_simplejwt.authentication.JWTAuthentication',
    ),
    'DEFAULT_PERMISSION_CLASSES': (
        'rest_framework.permissions.IsAuthenticated',
    ),
    'DEFAULT_FILTER_BACKENDS': (
        'django_filters.rest_framework.DjangoFilterBackend',
    ),
    'DEFAULT_PAGINATION_CLASS': 'rest_framework.pagination.PageNumberPagination',
    'PAGE_SIZE': 10,
}

# JWT settings
SIMPLE_JWT = {
    'ACCESS_TOKEN_LIFETIME': timedelta(minutes=60),
    'REFRESH_TOKEN_LIFETIME': timedelta(days=1),
    'ROTATE_REFRESH_TOKENS': False,
    'BLACKLIST_AFTER_ROTATION': True,
    'ALGORITHM': 'HS256',
    'SIGNING_KEY': SECRET_KEY,
    'VERIFYING_KEY': None,
    'AUTH_HEADER_TYPES': ('Bearer',),
    'USER_ID_FIELD': 'id',
    'USER_ID_CLAIM': 'user_id',
    'AUTH_TOKEN_CLASSES': ('rest_framework_simplejwt.tokens.AccessToken',),
    'TOKEN_TYPE_CLAIM': 'token_type',
}

# Swagger settings
SWAGGER_SETTINGS = {
    'SECURITY_DEFINITIONS': {
        'Bearer': {
            'type': 'apiKey',
            'name': 'Authorization',
            'in': 'header'
        }
    }
}

# CORS settings
CORS_ALLOW_ALL_ORIGINS = True
CORS_ALLOW_CREDENTIALS = True

# Prometheus settings
PROMETHEUS_EXPORT_MIGRATIONS = False

# Email settings
EMAIL_BACKEND = 'django_mailgun_provider.backends.MailgunAPIBackend'
MAILGUN_API_KEY = os.environ.get('MAILGUN_API_KEY', '')
MAILGUN_SENDER_DOMAIN = os.environ.get('MAILGUN_SENDER_DOMAIN', 'mg.example.com')
DEFAULT_FROM_EMAIL = os.environ.get('DEFAULT_FROM_EMAIL', 'noreply@example.com')

# Google Chat Webhook
GOOGLE_CHAT_WEBHOOK_URL = os.environ.get('GOOGLE_CHAT_WEBHOOK_URL', '')

# Template directories
TEMPLATES[0]['DIRS'] = [os.path.join(BASE_DIR, 'admin_users', 'templates')]

# Logging configuration
LOGGING_LEVEL = os.environ.get('LOGGING_LEVEL', 'INFO')
LOGGING = {
    'version': 1,
    'disable_existing_loggers': False,
    'formatters': {
        'verbose': {
            'format': '{levelname} {asctime} {module} {process:d} {thread:d} {message}',
            'style': '{',
        },
        'simple': {
            'format': '{levelname} {message}',
            'style': '{',
        },
    },
    'handlers': {
        'console': {
            'level': LOGGING_LEVEL,
            'class': 'logging.StreamHandler',
            'stream': sys.stdout,
            'formatter': 'verbose',
        },
    },
    'root': {
        'handlers': ['console'],
        'level': LOGGING_LEVEL,
    },
    'loggers': {
        'django': {
            'handlers': ['console'],
            'level': LOGGING_LEVEL,
            'propagate': False,
        },
        'django.db.backends': {
            'handlers': ['console'],
            'level': LOGGING_LEVEL,
            'propagate': False,
        },
        'authentication': {
            'handlers': ['console'],
            'level': 'DEBUG',
            'propagate': False,
        },
        'admin_users': {
            'handlers': ['console'],
            'level': 'DEBUG',
            'propagate': False,
        },
    },
}

# MongoDB settings
MONGODB_URI = os.environ.get('MONGODB_URI', 'mongodb://mongodb:27017/radius_accounting')
try:
    MONGODB_CLIENT = pymongo.MongoClient(MONGODB_URI)
    MONGODB_DB = MONGODB_CLIENT.get_database()
except pymongo.errors.ConnectionFailure:
    MONGODB_CLIENT = None
    MONGODB_DB = None
    print("Warning: Could not connect to MongoDB. Accounting features will not be available.")
