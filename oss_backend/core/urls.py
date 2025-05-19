from django.contrib import admin
from django.urls import path, include
from django.http import HttpResponse
from django.conf import settings
from django.conf.urls.static import static
from rest_framework import permissions
from drf_yasg.views import get_schema_view
from drf_yasg import openapi


# Health check view
def health_check(request):
    return HttpResponse(status=200)

# Schema view for Swagger documentation
schema_view = get_schema_view(
    openapi.Info(
        title="API Documentation",
        default_version='v1',
        description="API documentation for the project",
        terms_of_service="https://www.example.com/terms/",
        contact=openapi.Contact(email="contact@example.com"),
        license=openapi.License(name="BSD License"),
    ),
    public=True,
    permission_classes=(permissions.AllowAny,),
)

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
    path('api/auth/', include('authentication.urls')),
    path('api/admin-users/', include('admin_users.urls')),
    path('api/nas/', include('nas.urls')),
    path('api/settings/', include('settings_app.urls')),
    path('api/users/', include('users.urls')),
    path('api/accounting/', include('accounting.urls')),
    path('api/radius/', include('radius.urls')),
    path('api/shared/', include('shared.urls')),
]

# Serve static files in development
if settings.DEBUG:
    urlpatterns += static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
