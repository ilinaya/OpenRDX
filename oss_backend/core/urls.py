from django.contrib import admin
from django.urls import path, include
from django.http import HttpResponse
from django.conf import settings
from django.conf.urls.static import static
from rest_framework import permissions
from drf_yasg.views import get_schema_view
from drf_yasg import openapi
from nas.views import NasViewSet


# Health check view
def health_check(request):
    return HttpResponse(status=200)

# Schema view for Swagger documentation
schema_view = get_schema_view(
    openapi.Info(
        title="OpenRDX Backend API Documentation",
        default_version='v1',
        description="API documentation for the OpenRDX backend",
        terms_of_service="https://openrdx.org/",
        contact=openapi.Contact(email="alexey@openrdx.org"),
        license=openapi.License(name="MIT License"),
    ),
    public=True,
    permission_classes=(permissions.AllowAny,),
)

# Create a view for POST to /api/nas (without trailing slash) to prevent 301 redirect
nas_create_view = NasViewSet.as_view({'post': 'create'})

urlpatterns = [
    path('admin/', admin.site.urls),

    # Health check endpoint
    path('health/', health_check, name='health_check'),

    # Swagger documentation
    path('swagger/', schema_view.with_ui('swagger', cache_timeout=0), name='schema-swagger-ui'),
    path('redoc/', schema_view.with_ui('redoc', cache_timeout=0), name='schema-redoc'),

    # Prometheus metrics
    path('', include('django_prometheus.urls')),

    # API endpoints
    path('auth/', include('authentication.urls')),
    path('admin-users/', include('admin_users.urls')),
    # Handle POST to /api/nas (without trailing slash) before the nas/ route to prevent 301 redirect
    path('nas', nas_create_view, name='nas-create-no-slash'),
    path('nas/', include('nas.urls')),
    path('users/', include('users.urls')),
    path('accounting/', include('accounting.urls')),
    path('radius/', include('radius.urls')),
    path('radsec/', include('radsec.urls')),
    path('shared/', include('shared.urls')),
    path('api-keys/', include('api_keys.urls')),
]

if settings.DEBUG:
    urlpatterns += static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
