import {Routes} from '@angular/router';
import {DevicesComponent} from './devices.component';
import {NasListComponent} from './components/nas-list/nas-list.component';
import {NasFormComponent} from './components/nas-form/nas-form.component';
import {NasDetailComponent} from './components/nas-detail/nas-detail.component';
import {NasGroupListComponent} from './components/nas-group-list/nas-group-list.component';
import {NasGroupFormComponent} from './components/nas-group-form/nas-group-form.component';

export const routes: Routes = [
  {
    path: '',
    component: DevicesComponent,
    children: [
      {path: '', redirectTo: 'nas', pathMatch: 'full'},
      {path: 'nas', component: NasListComponent},
      {path: 'nas/new', component: NasFormComponent},
      {path: 'nas/:id', component: NasDetailComponent},
      {path: 'nas/:id/edit', component: NasFormComponent},
      {path: 'groups', component: NasGroupListComponent},
      {path: 'groups/new', component: NasGroupFormComponent},
      {path: 'groups/:id/edit', component: NasGroupFormComponent},
    ],
  },
];
