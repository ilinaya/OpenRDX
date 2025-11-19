from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import AdminUserViewSet, AdminGroupViewSet, set_password, reset_password

# Create a router and register our viewsets with it
router = DefaultRouter()
router.register(r'users', AdminUserViewSet, basename='admin-user')
router.register(r'groups', AdminGroupViewSet, basename='admin-group')

# The API URLs are now determined automatically by the router
urlpatterns = [
    path('', include(router.urls)),
    path('users/list/', AdminUserViewSet.as_view({'get': 'list_all'}), name='admin-user-list-all'),
    path('groups/list/', AdminGroupViewSet.as_view({'get': 'list_all'}), name='admin-group-list-all'),

    path('me/', AdminUserViewSet.as_view({'get': 'me', 'patch': 'me'}), name='admin-user-me'),
    path('change-password/', AdminUserViewSet.as_view({'post': 'change_password'}), name='change-password'),
    path('set-password/', set_password, name='set-password'),
    path('reset-password/', reset_password, name='reset-password'),
]
