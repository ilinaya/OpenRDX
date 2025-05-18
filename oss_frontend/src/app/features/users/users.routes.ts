import {Routes} from '@angular/router';
import {UserGroupListComponent} from '../settings/components/user-group-list/user-group-list.component';
import {UserGroupFormComponent} from '../settings/components/user-group-form/user-group-form.component';

export const routes: Routes = [
  {
    path: 'groups',
    children: [
      {path: '', component: UserGroupListComponent},
      {path: 'create', component: UserGroupFormComponent},
      {path: ':id/edit', component: UserGroupFormComponent},
    ],
  },
];
