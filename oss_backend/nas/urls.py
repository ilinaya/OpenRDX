from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import NasViewSet, NasGroupViewSet, VendorsViewSet

router = DefaultRouter()
router.register(r'nas', NasViewSet)
router.register(r'groups', NasGroupViewSet)

router.register(r'vendors', VendorsViewSet)

# Create a view for POST to /api/nas (without trailing slash) to avoid 301 redirect
# This view only handles POST requests and delegates to NasViewSet.create
nas_create_view = NasViewSet.as_view({'post': 'create'})

urlpatterns = [
    # Handle POST to /api/nas (without trailing slash) to avoid 301 redirect
    # This must come before router.urls to catch POST requests first
    path('', nas_create_view, name='nas-create-no-slash'),
    path('', include(router.urls)),
]