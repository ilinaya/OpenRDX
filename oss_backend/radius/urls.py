from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import (
    AuthAttributeGroupViewSet,
    RadiusAttributeViewSet,
    UserNasRelationshipViewSet,
    SecretViewSet
)

router = DefaultRouter()
router.register(r'attribute-groups', AuthAttributeGroupViewSet)
router.register(r'attributes', RadiusAttributeViewSet)
router.register(r'user-nas', UserNasRelationshipViewSet)
router.register(r'secrets', SecretViewSet)


urlpatterns = [
    path('', include(router.urls)),
    path('attribute-groups/list/', AuthAttributeGroupViewSet.as_view({'get': 'list_all'}), name='attribute-group-list-all'),
    path('secrets/list/', SecretViewSet.as_view({'get': 'list_all'}), name='secret-list-all'),
]
