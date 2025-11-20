from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import ApiKeyViewSet

router = DefaultRouter()
router.register(r'', ApiKeyViewSet, basename='apikey')

urlpatterns = [
    path('', include(router.urls)),
]

