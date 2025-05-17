from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import SettingViewSet

app_name = 'settings_app'

router = DefaultRouter()
router.register('', SettingViewSet)

urlpatterns = [
    path('', include(router.urls)),
]