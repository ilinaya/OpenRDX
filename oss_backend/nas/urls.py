from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import NasViewSet, NasGroupViewSet, VendorsViewSet

router = DefaultRouter()
router.register(r'nas', NasViewSet)
router.register(r'groups', NasGroupViewSet)

router.register(r'vendors', VendorsViewSet)

urlpatterns = [
    path('', include(router.urls)),
]