import {Routes} from '@angular/router';
import {SettingsComponent} from './settings.component';
import {AdminListComponent} from './components/admin-list/admin-list.component';
import {AdminFormComponent} from './components/admin-form/admin-form.component';
import {AdminDetailComponent} from './components/admin-detail/admin-detail.component';
import {AdminGroupListComponent} from './components/admin-group-list/admin-group-list.component';
import {AdminGroupFormComponent} from './components/admin-group-form/admin-group-form.component';
import {AdminGroupDetailComponent} from './components/admin-group-detail/admin-group-detail.component';
import {SecretListComponent} from './components/secret-list/secret-list.component';
import {SecretFormComponent} from './components/secret-form/secret-form.component';
import {SecretDetailComponent} from './components/secret-detail/secret-detail.component';
import {VendorListComponent} from './components/vendor-list/vendor-list.component';
import {VendorFormComponent} from './components/vendor-form/vendor-form.component';
import {VendorDetailComponent} from './components/vendor-detail/vendor-detail.component';
import {ChangePasswordComponent} from './components/change-password/change-password.component';
import {AttributeGroupListComponent} from './components/attribute-group-list/attribute-group-list.component';
import {AttributeGroupFormComponent} from './components/attribute-group-form/attribute-group-form.component';

export const routes: Routes = [
  {
    path: '',
    component: SettingsComponent,
    children: [
      {
        path: 'admins',
        children: [
          {path: '', component: AdminListComponent},
          {path: 'new', component: AdminFormComponent},
          {path: ':id', component: AdminDetailComponent},
          {path: ':id/edit', component: AdminFormComponent},
        ],
      },
      {
        path: 'groups',
        children: [
          {path: '', component: AdminGroupListComponent},
          {path: 'new', component: AdminGroupFormComponent},
          {path: ':id', component: AdminGroupDetailComponent},
          {path: ':id/edit', component: AdminGroupFormComponent},
        ],
      },
      {
        path: 'secrets',
        children: [
          {path: '', component: SecretListComponent},
          {path: 'new', component: SecretFormComponent},
          {path: ':id', component: SecretDetailComponent},
          {path: ':id/edit', component: SecretFormComponent},
        ],
      },
      {
        path: 'vendors',
        children: [
          {path: '', component: VendorListComponent},
          {path: 'new', component: VendorFormComponent},
          {path: ':id', component: VendorDetailComponent},
          {path: ':id/edit', component: VendorFormComponent},
        ],
      },
      {
        path: 'change-password',
        component: ChangePasswordComponent,
      },
      {
        path: 'attribute-groups',
        children: [
          {path: '', component: AttributeGroupListComponent},
          {path: 'create', component: AttributeGroupFormComponent},
          {path: ':id/edit', component: AttributeGroupFormComponent},
        ],
      },
      {path: '', redirectTo: 'admins', pathMatch: 'full'},
    ],
  },
];
