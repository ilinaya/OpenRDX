from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import TimezoneViewSet

router = DefaultRouter()
router.register(r'timezones', TimezoneViewSet)

urlpatterns = [
    path('', include(router.urls)),
] 