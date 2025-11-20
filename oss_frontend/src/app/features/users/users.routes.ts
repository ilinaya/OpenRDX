import {Routes} from '@angular/router';
import {UsersComponent} from "./users.component";
import {UserListComponent} from "./components/user-list/user-list.component";
import {UserFormComponent} from "./components/user-form/user-form.component";
import {UserDetailComponent} from "./components/user-detail/user-detail.component";
import {UserGroupFormComponent} from "./components/user-group-form/user-group-form.component";
import {UserGroupListComponent} from "./components/user-group-list/user-group-list.component";
import {UserGroupDetailComponent} from "./components/user-group-detail/user-group-detail.component";

export const routes: Routes = [
  {
    path: '',
    component: UsersComponent,
    children: [
      {path: '', redirectTo: 'users', pathMatch: 'full'},
      {path: 'users', component: UserListComponent},
      {path: 'users/new', component: UserFormComponent},
      {path: 'users/:id', component: UserDetailComponent},
      {path: 'users/:id/edit', component: UserFormComponent},
      {path: 'groups', component: UserGroupListComponent},
      {path: 'groups/new', component: UserGroupFormComponent},
      {path: 'groups/:id', component: UserGroupDetailComponent},
      {path: 'groups/:id/edit', component: UserGroupFormComponent},
    ],
  },
];
